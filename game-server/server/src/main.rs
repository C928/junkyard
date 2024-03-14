use anyhow::bail;
use async_channel::{Receiver, Sender};
use game_server::action_check::{player_attack, reach_destination};
use game_server::response::packet_sizes::*;
use game_server::response::*;
use net_utils::character::{Character, CharacterClass};
use net_utils::map::Point;
use net_utils::packet::game_data_code::*;
use net_utils::packet::request_codes::*;
use net_utils::packet::status_codes::*;
use rand::prelude::SliceRandom;
use rand::{thread_rng, Rng};
use redis::{pipe, Commands, Connection, IntoConnectionInfo};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;
use std::sync::Arc;
use std::time::Duration;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::Mutex;
use tokio::task;
use uuid::Uuid;

async fn write_packet_from_json(stream: &mut TcpStream, json: &String) {
    let bytes = json.as_bytes();
    let packet_size = &bytes.len().to_be_bytes()[6..];
    stream.write(packet_size).await.unwrap();
    stream.write(bytes).await.unwrap();
}

async fn write_packet_from_code(stream: &mut TcpStream, code: u64, size: u16) {
    let packet_size = &size.to_be_bytes();
    let json = json!({ "status": code }).to_string();
    stream.write(packet_size).await.unwrap();
    stream.write(json.as_bytes()).await.unwrap();
}

async fn read_packet(stream: &mut TcpStream) -> Value {
    let mut packet_size = [0; 2];
    stream.read_exact(&mut packet_size).await.unwrap();
    let size = u16::from_be_bytes(packet_size);

    let mut packet_vec = (0..size).map(|_| 0).collect::<Vec<_>>();
    stream.read_exact(&mut packet_vec).await.unwrap();

    serde_json::from_slice(&*packet_vec).unwrap()
}

/*
fn try_read_packet(stream: &mut TcpStream) -> Option<Value> {
    let mut packet_size = [0; 2];
    match stream.try_read(&mut packet_size) {
        Ok(0) | Err(_) => return None,
        _ => (),
    }
    let size = u16::from_be_bytes(packet_size);

    let mut packet_vec = (0..size).map(|_| 0).collect::<Vec<_>>();
    match stream.try_read(&mut packet_vec) {
        Ok(0) | Err(_) => return None,
        _ => (),
    }

    Some(serde_json::from_slice(&*packet_vec).unwrap())
}
*/

#[derive(Deserialize, Serialize)]
struct PlayerInfos {
    pseudo: String,
    hosting: u8,
}

type Channel = (Sender<String>, Receiver<String>);

struct GameChannel {
    channel: Channel,
    player_count: u8,
}
impl GameChannel {
    fn new(channel: Channel) -> Self {
        Self {
            channel,
            player_count: 1,
        }
    }

    async fn broadcast(&self, json: String) {
        // send packet to every player in the game except the one on the current tokio thread
        for _ in 0..self.player_count - 1 {
            self.channel.0.send(json.clone()).await.unwrap();
            task::yield_now().await;
        }
    }
}

struct State {
    // hashmap storing game tokens with associated channels to communicate between tokio threads
    state: Mutex<HashMap<String, GameChannel>>,
}
impl State {
    fn new() -> Self {
        Self {
            state: Mutex::new(HashMap::new()),
        }
    }
}

//todo: limit number of trees, rocks and water pools
fn generate_random_map(height: u8, width: u8) -> String {
    let mut rng = thread_rng();
    let mut obstacle_cnt = 0;
    let mut map: String = std::iter::repeat('0').take(width as usize).collect();
    map.push('\n');
    for h in 0..(height - 2) {
        for w in 0..width {
            let o = ['0', 'R', 'W', 'T'][rng.gen_range(0..=3)];
            map.push(if obstacle_cnt != width {
                if o != '0' {
                    obstacle_cnt += 1;
                }
                o
            } else {
                '0'
            });
        }
        map.push('\n');
        obstacle_cnt = 0;
    }
    let s: String = std::iter::repeat('0').take((width+1) as usize).collect();
    map.push_str(&s);
    map

    /*
    (0..height * width)
        .enumerate()
        .map(|(i, _)| {
            if i < width as usize || i >= ((height - 1) * width) as usize {
                '0'
            } else {
                // nothing, rock, water, tree
                ['0', 'R', 'W', 'T'][rng.gen_range(0..=3)]
            }
        })
        .collect()
     */
}

async fn verify_player_token(
    stream: &mut TcpStream,
    redis_con: &mut Connection,
    json_req: &Value,
) -> anyhow::Result<String> {
    let player_token = match json_req["player_token"].as_str() {
        Some(p_token) => {
            if p_token.len() != 36
                || !redis_con
                    .hexists::<&str, &str, bool>("player", p_token)
                    .unwrap()
            {
                pub const GM_AL_START_SIZE: u16 = DEF;
                write_packet_from_code(stream, ERR_INV_PL_TOK, INV_PL_TOK_SIZE).await;
                bail!("")
            }
            p_token
        }
        None => {
            write_packet_from_code(stream, ERR_MAL_REQ, MAL_REQ_SIZE).await;
            bail!("")
        }
    };

    Ok(player_token.to_owned())
}

async fn verify_game_token(
    stream: &mut TcpStream,
    redis_con: &mut Connection,
    json_req: &Value,
) -> anyhow::Result<String> {
    let game_token = match json_req["game_token"].as_str() {
        Some(g_token) => {
            if g_token.len() != 36
                || !redis_con
                    .sismember::<&str, &str, bool>("game", g_token)
                    .unwrap()
            {
                write_packet_from_code(stream, ERR_INV_GM_TOK, INV_GM_TOK_SIZE).await;
                bail!("");
            }
            g_token
        }
        None => {
            write_packet_from_code(stream, ERR_MAL_REQ, MAL_REQ_SIZE).await;
            bail!("");
        }
    };

    Ok(game_token.to_owned())
}

async fn player_creation(stream: &mut TcpStream, redis_con: &mut Connection, json_req: Value) {
    let pseudo = match json_req["pseudo"].as_str() {
        Some(p) => {
            if p.len() > 32 || !p.chars().all(char::is_alphanumeric) {
                write_packet_from_code(stream, ERR_INV_PSEUD, INV_PSEUD_SIZE).await;
                return;
            }
            p
        }
        None => {
            write_packet_from_code(stream, ERR_MAL_REQ, MAL_REQ_SIZE).await;
            return;
        }
    };

    loop {
        let player_token = Uuid::new_v4().to_string();
        if !redis_con
            .hexists::<&str, &String, bool>("player", &player_token)
            .unwrap()
        {
            let player_infos = json!({"pseudo": pseudo, "hosting": 0}).to_string();
            if !redis_con
                .hset::<&str, &String, String, bool>("player", &player_token, player_infos)
                .unwrap()
            {
                //todo: return internal server error
            }
            let json = PlayerCreation::json_string(&player_token).unwrap();
            write_packet_from_json(stream, &json).await;
            //todo: wait for client response (to avoid inserting player in database if write_packet_from_json fails)
            return;
        }
    }
}

async fn game_creation(
    state: &Arc<State>,
    stream: &mut TcpStream,
    mut redis_con: &mut Connection,
    json_req: Value,
) -> Channel {
    let player_token = match verify_player_token(stream, redis_con, &json_req).await {
        Ok(p_token) => p_token,
        Err(_) => panic!(),
    };

    let mut player_infos: PlayerInfos = serde_json::from_str(
        &*redis_con
            .hget::<&str, &String, String>("player", &player_token)
            .unwrap(),
    )
    .unwrap();
    if player_infos.hosting == 1 {
        //todo: write error (already hosting)
        panic!();
    }

    let mut game_token;
    loop {
        game_token = Uuid::new_v4().to_string();
        let game_info_hash_key = format!("game_info:{}", game_token);
        let game_player_hash_key = format!("game_player:{}", game_token);
        let ret: Option<redis::Value> = redis::transaction(
            &mut redis_con,
            &["game", "player", &game_info_hash_key, &game_player_hash_key],
            |redis_con, pipe| {
                if redis_con
                    .sismember::<&str, &str, bool>("game_info", &game_token)
                    .unwrap()
                {
                    return Ok(None);
                }
                //todo: expiration for game keys ?
                //todo: return internal server error if failing?

                let mut rng = thread_rng();
                let mut vec = vec!['1', '2'];
                vec.shuffle(&mut rng);
                let turn = String::from_iter(vec);
                player_infos.hosting = 1;
                pipe.sadd("game", &game_token)
                    .ignore()
                    .hset_multiple(
                        &game_info_hash_key,
                        &[
                            ("started", "0"),
                            ("host_player", &player_token),
                            ("player_count", "1"),
                            ("map", &*generate_random_map(5, 10)),
                            ("turn", &turn),
                        ],
                    )
                    .ignore()
                    .hset(
                        "player",
                        &player_token,
                        serde_json::to_string(&player_infos).unwrap(),
                    )
                    .ignore()
                    .hset(&game_player_hash_key, &player_token, "1")
                    .ignore()
                    .query(redis_con)
            },
        )
        .unwrap();

        if ret.is_none() {
            // game token already exists (very rare but still a possibility)
            continue;
        }
        break;
    }
    //todo: internal server error if on of tuple field is false? and delete "true" field from redis?

    let json = GameCreation::json_string(&game_token).unwrap();
    write_packet_from_json(stream, &json).await;

    //{
    let channel: Channel = async_channel::bounded(50);
    let c_clone: Channel = channel.clone();
    let game_channel = GameChannel::new(channel);
    state.state.lock().await.insert(game_token, game_channel);
    c_clone
    //}
}

async fn game_joining(
    state: &Arc<State>,
    mut stream: &mut TcpStream,
    redis_con: &mut Connection,
    json_req: Value,
) -> Channel {
    //todo: tmp, todo first
    //let _ = state.game_joining_lock.lock().await;

    //todo: move inside transaction
    let player_token = match verify_player_token(stream, redis_con, &json_req).await {
        Ok(p_token) => p_token,
        Err(_) => panic!(),
    };

    let game_token = match verify_game_token(stream, redis_con, &json_req).await {
        Ok(g_token) => g_token,
        Err(_) => panic!(),
    };

    let game_info_hash_key = format!("game_info:{game_token}");
    let game_player_hash_key = format!("game_player:{}", game_token);

    //todo: transaction
    let mut transaction_error = None;
    let mut player_infos: Option<PlayerInfos> = None;
    let mut player_vec = None;
    let ret: Option<redis::Value> = redis::transaction(
        redis_con,
        &[&game_info_hash_key, &game_player_hash_key],
        |redis_con, pipe| {
            let started: bool = redis_con.hget(&game_info_hash_key, "started").unwrap();
            if started {
                transaction_error = Some((ERR_GM_AL_START, GM_AL_START_SIZE));
                return Ok(None);
            }

            if redis_con
                .hexists(&game_player_hash_key, &player_token)
                .unwrap()
            {
                //todo: write error (player already in the game)
                return Ok(None);
            }

            let mut player_count: u8 = redis_con.hget(&game_info_hash_key, "player_count").unwrap();

            if player_count == 2 {
                transaction_error = Some((ERR_GM_FULL, GM_FULL_SIZE));
                return Ok(None);
            }

            player_count += 1;
            redis_con
                .hset::<&String, &str, u8, bool>(&game_info_hash_key, "player_count", player_count)
                .unwrap();

            let ret: u8 = redis_con
                .hset(&game_player_hash_key, &player_token, player_count)
                .unwrap();

            if ret != 1 {
                panic!();
            }
            player_infos = Some(
                serde_json::from_str(
                    &*redis_con
                        .hget::<&str, &String, String>("player", &player_token)
                        .unwrap(),
                )
                .unwrap(),
            );

            let mut p_vec = vec![];
            let player_hm: HashMap<String, String> =
                redis_con.hgetall(&game_player_hash_key).unwrap();
            for (p_token, character) in player_hm {
                let p_infos: PlayerInfos = serde_json::from_str(
                    &*redis_con
                        .hget::<&str, &String, String>("player", &p_token)
                        .unwrap(),
                )
                .unwrap();
                p_vec.push([
                    player_count.to_string(),
                    p_infos.pseudo,
                    character,
                    p_infos.hosting.to_string(),
                ]);
            }
            player_vec = Some(p_vec);

            pipe.query(redis_con)
        },
    )
    .unwrap();

    if ret.is_none() {
        panic!();
    }

    if let Some(error_code) = transaction_error {
        write_packet_from_code(stream, error_code.0, error_code.1).await;
        panic!();
    }

    let player_infos = player_infos.unwrap();
    let json = GameJoining::json_string(&player_infos.pseudo, player_vec.unwrap()).unwrap();

    let mut hm = state.state.lock().await;
    let mut game_channel = hm.remove(&game_token).unwrap();
    let c_clone: Channel = game_channel.channel.clone();
    game_channel.player_count += 1;

    write_packet_from_json(stream, &json).await;
    game_channel.broadcast(json).await;

    hm.insert(game_token, game_channel);
    c_clone
}

async fn character_choosing(
    state: &Arc<State>,
    mut stream: &mut TcpStream,
    redis_con: &mut Connection,
    json_req: Value,
) {
    let player_token = match verify_player_token(stream, redis_con, &json_req).await {
        Ok(p_token) => p_token,
        Err(_) => panic!(),
    };

    let game_token = match verify_game_token(stream, redis_con, &json_req).await {
        Ok(g_token) => g_token,
        Err(_) => panic!(),
    };

    let game_player_hash_key = format!("game_player:{}", game_token);
    let player_joined: bool = redis_con
        .hexists(&game_player_hash_key, &player_token)
        .unwrap();
    if !player_joined {
        write_packet_from_code(stream, ERR_GM_NOT_JOIN, GM_NOT_JOIN_SIZE).await;
        return;
    }

    let started: bool = redis_con
        .hget(format!("game_info:{}", game_token), "started")
        .unwrap();
    if started {
        write_packet_from_code(&mut stream, ERR_GM_AL_START, GM_AL_START_SIZE).await;
        return;
    }

    match json_req["character"].as_str() {
        Some(character) => {
            let character_class = match CharacterClass::new(character) {
                Some(c) => c,
                None => {
                    write_packet_from_code(&mut stream, ERR_MAL_REQ, MAL_REQ_SIZE).await;
                    return;
                }
            };

            let stats = character_class.get_stats();
            let player_num: String = redis_con
                .hget(&game_player_hash_key, &player_token)
                .unwrap();
            let game_player_infos =
                GamePlayerInfos::json_string(player_num, character.into(), stats).unwrap();

            redis_con
                .hset::<String, &str, String, bool>(
                    game_player_hash_key,
                    &player_token,
                    game_player_infos,
                )
                .unwrap();
            let player_infos: PlayerInfos = serde_json::from_str(
                &*redis_con
                    .hget::<&str, &String, String>("player", &player_token)
                    .unwrap(),
            )
            .unwrap();

            let lock = state.state.lock().await;
            let game_channel = lock.get(&game_token).unwrap();
            let json = CharacterChoosing::json_string(&player_infos.pseudo, character).unwrap();
            write_packet_from_json(stream, &json).await;
            game_channel.broadcast(json).await;
        }
        None => {
            write_packet_from_code(&mut stream, ERR_MAL_REQ, MAL_REQ_SIZE).await;
            return;
        }
    }
}

async fn game_starting(
    state: &Arc<State>,
    stream: &mut TcpStream,
    redis_con: &mut Connection,
    json_req: Value,
) {
    let player_token = match verify_player_token(stream, redis_con, &json_req).await {
        Ok(p_token) => p_token,
        Err(_) => panic!(),
    };

    let game_token = match verify_game_token(stream, redis_con, &json_req).await {
        Ok(g_token) => g_token,
        Err(_) => panic!(),
    };

    let game_info_hash_key = format!("game_info:{game_token}");
    let game_info: HashMap<String, String> = redis_con.hgetall(&game_info_hash_key).unwrap();
    if game_info.get("started") == Some(&"0".to_owned()) {
        // only the host player can start the game
        if game_info.get("host_player") == Some(&player_token) {
            let player_count = game_info.get("player_count");
            if player_count == Some(&"2".to_owned()) {
                redis_con
                    .hset::<&String, &str, &str, bool>(&game_info_hash_key, "started", "1")
                    .unwrap();

                let mut turn: String = redis_con.hget(&game_info_hash_key, "turn").unwrap();
                let player_turn = String::from(&turn[0..1]);

                let mut map: String = redis_con.hget(&game_info_hash_key, "map").unwrap();
                let len = map.len();
                //todo first: uncomment (was commented to test with client)
                //let v = ['1', '2'];
                //v.shuffle(&mut thread_rng());
                //map = format!("{}{}{}", v[0], &map[1..len-1], v[1]);
                map = format!("1{}2", &map[1..len - 2]);
                redis_con
                    .hset::<&String, &str, &String, bool>(&game_info_hash_key, "map", &map)
                    .unwrap();
                let json = GameStarting::json_string(player_turn, map).unwrap();

                let lock = state.state.lock().await;
                let game_channel = lock.get(&game_token).unwrap();
                write_packet_from_json(stream, &json).await;
                game_channel.broadcast(json).await;
            } else {
                write_packet_from_code(stream, ERR_GM_NOT_FULL, GM_NOT_FULL_SIZE).await;
            }
        }
    } else {
        write_packet_from_code(stream, ERR_GM_AL_START, GM_AL_START_SIZE).await;
    }
}

async fn game_data_parsing(
    state: &Arc<State>,
    stream: &mut TcpStream,
    redis_con: &mut Connection,
    json_req: Value,
) {
    let gm_data_type = match json_req["gm_code"].as_u64() {
        Some(GM_DATA_MOV) => {
            let target = match json_req["target"].as_array() {
                Some(t) => Point(t[0].as_u64().unwrap() as i16, t[1].as_u64().unwrap() as i16),
                None => {
                    write_packet_from_code(stream, ERR_MAL_REQ, MAL_REQ_SIZE).await;
                    return;
                }
            };

            (GM_DATA_MOV, target)
        }
        Some(GM_DATA_ATK) => {
            let target = match json_req["target"].as_array() {
                Some(t) => Point(t[0].as_u64().unwrap() as i16, t[1].as_u64().unwrap() as i16),
                None => {
                    write_packet_from_code(stream, ERR_MAL_REQ, MAL_REQ_SIZE).await;
                    return;
                }
            };

            (GM_DATA_ATK, target)
        }
        Some(GM_DATA_SKIP) => (GM_DATA_SKIP, Point(0, 0)),
        None | Some(_) => {
            write_packet_from_code(stream, ERR_MAL_REQ, MAL_REQ_SIZE).await;
            return;
        }
    };

    let player_token = match verify_player_token(stream, redis_con, &json_req).await {
        Ok(p_token) => p_token,
        Err(_) => panic!(),
    };

    let game_token = match verify_game_token(stream, redis_con, &json_req).await {
        Ok(g_token) => g_token,
        Err(_) => panic!(),
    };

    let game_info_hash_key = format!("game_info:{}", game_token);
    let started: bool = redis_con.hget(&game_info_hash_key, "started").unwrap();
    if !started {
        write_packet_from_code(stream, ERR_GM_NOT_START, GM_NOT_START_SIZE).await;
        return;
    }

    let mut map: String = redis_con.hget(&game_info_hash_key, "map").unwrap();
    let mut turn: String = redis_con.hget(&game_info_hash_key, "turn").unwrap();
    let player_num = String::from(&turn[0..1]);
    turn = format!("{}{}", &turn[1..], &turn[0..1]);

    let mut gm_player_infos = None;
    let infos_vec = redis_con
        .hgetall::<String, HashMap<String, String>>(format!("game_player:{}", game_token))
        .unwrap();
    let mut enemy_gp_infos_vec: Vec<GamePlayerInfos> = vec![];
    for (p_token, gm_p_infos_str) in infos_vec {
        let gm_p_infos: GamePlayerInfos = serde_json::from_str(&*gm_p_infos_str).unwrap();
        if p_token == player_token {
            gm_player_infos = Some(gm_p_infos);
            continue;
        }
        enemy_gp_infos_vec.push(gm_p_infos);
    }

    let gm_player_infos = gm_player_infos.unwrap();
    if player_num != gm_player_infos.player_num {
        //write_packet_from_code(stream, ERR_);
        // not the player turn
        return;
    }

    let character = match Character::from_str(&gm_player_infos.character) {
        Some(c) => c,
        None => {
            write_packet_from_code(stream, ERR_MAL_REQ, MAL_REQ_SIZE).await;
            return;
        }
    };

    //todo: movement/attack verification on map
    let stats = character.class.get_stats();
    let ret_fields = match gm_data_type.0 {
        GM_DATA_MOV => {
            if reach_destination(
                &mut map,
                gm_player_infos.player_num,
                gm_data_type.1,
                character.class.get_stats().2,
            ) {
                Some((GM_DATA_MOV, ("".into(), 0)))
            } else {
                None
            }
        }
        GM_DATA_ATK => {
            if let Some(enemy_update) = player_attack(
                &mut map,
                gm_player_infos.player_num,
                enemy_gp_infos_vec,
                gm_data_type.1,
                (stats.0, stats.3),
            ) {
                Some((GM_DATA_ATK, enemy_update))
            } else {
                None
            }
        }
        GM_DATA_SKIP => Some((GM_DATA_SKIP, ("".into(), 0))),
        _ => None,
    };

    if let Some(ret_fields) = ret_fields {
        redis_con
            .hset::<&String, &str, &String, bool>(&game_info_hash_key, "map", &map)
            .unwrap();
        redis_con
            .hset::<String, &str, &String, bool>(game_info_hash_key, "turn", &turn)
            .unwrap();
        let json =
            GameData::json_string(ret_fields.0, turn[0..1].to_string(), player_num, map, ret_fields.1).unwrap();

        let lock = state.state.lock().await;
        let game_channel = lock.get(&game_token).unwrap();
        write_packet_from_json(stream, &json).await;
        game_channel.broadcast(json).await;
    } else {
        write_packet_from_code(stream, ERR_MAL_REQ, MAL_REQ_SIZE).await;
    }
}

async fn handle_player(
    state: Arc<State>,
    mut stream: TcpStream,
    redis_con: &mut Connection,
) -> anyhow::Result<()> {
    let mut json = read_packet(&mut stream).await;
    if let Some(PL_CREAT) = json["request_type"].as_u64() {
        player_creation(&mut stream, redis_con, json).await;
    }

    json = read_packet(&mut stream).await;
    let mut _is_host = false;
    let request_type = json["request_type"].as_u64();
    let channel = if let Some(GM_CREAT) = request_type {
        _is_host = true;
        game_creation(&state, &mut stream, redis_con, json).await
    } else if let Some(GM_JOIN) = request_type {
        game_joining(&state, &mut stream, redis_con, json).await
    } else {
        return Ok(());
    };

    loop {
        tokio::select! {
            Ok(packet) = channel.1.recv() => write_packet_from_json(&mut stream, &packet).await,

            json = read_packet(&mut stream) => {
                match json["request_type"].as_u64() {
                    Some(GM_DATA) => game_data_parsing(&state, &mut stream, redis_con, json).await,
                    Some(CHAR_CHOOSING) => character_choosing(&state, &mut stream, redis_con, json).await,
                    Some(GM_START) => game_starting(&state, &mut stream, redis_con, json).await,
                    Some(TERM_CON) => {
                        write_packet_from_code(&mut stream, TERM_CON, TERM_CON_SIZE).await;
                        break;
                    },
                    _ => {
                        write_packet_from_code(&mut stream, ERR_MAL_REQ, MAL_REQ_SIZE).await;
                        break;
                    },
                }
            },
        }
    }

    Ok(())
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let redis_client = redis::Client::open("redis://127.0.0.1:6379")?;
    if redis_client.get_connection().is_err() {
        eprintln!("redis instance not started");
        return Ok(());
    }
    let listener = TcpListener::bind("127.0.0.1:8000").await?;
    let state = Arc::new(State::new());

    while let Ok((stream, _addr)) = listener.accept().await {
        let state = Arc::clone(&state);
        let mut redis_con = redis_client.get_connection().unwrap();
        tokio::spawn(async move {
            handle_player(state, stream, &mut redis_con).await.unwrap();
        });
    }

    Ok(())
}

//todo: set ttl for tcp streams
//todo: try_write() or try_read() to avoid blocking the mutex
//todo: check redis queries concurrency
//todo: encryption for tcp (tls)
//todo: 1 channel for each conn ?

//todo: don't respond to certain requests (ex: player starting game but is not the host)
//todo: when game terminates, make player_infos hosting to false

//todo: manage request spamming from client
//todo: make net-utils a lib project
//todo: manage unwraps, panics, bails