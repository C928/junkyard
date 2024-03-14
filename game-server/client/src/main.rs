mod request;
mod test;

use crate::test::test_clients;
use net_utils::character::*;
use net_utils::map::{GameDataType, Point};
use net_utils::packet::game_data_code::*;
use net_utils::packet::{request_codes::*, status_codes::*};
use request::*;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::io::{stdin, Read, Write};
use std::net::TcpStream;
use std::sync::Arc;
use std::thread;
use std::thread::sleep;
use std::time::Duration;

fn read_packet(stream: &mut TcpStream) -> Vec<u8> {
    let mut packet_size = [0; 2];
    stream.read_exact(&mut packet_size).unwrap();
    let size = u16::from_be_bytes(packet_size);

    let mut packet_vec = (0..size).map(|_| 0).collect::<Vec<_>>();
    stream.read_exact(&mut packet_vec).unwrap();

    packet_vec
}

fn write_packet_from_json(stream: &mut TcpStream, json: String) {
    let bytes = json.as_bytes();
    let packet_size = &bytes.len().to_be_bytes()[6..];
    stream.write(packet_size).unwrap();
    stream.write(bytes).unwrap();
}

fn create_player(stream: &mut TcpStream, username: &str) -> String {
    let mut player_token = String::new();
    loop {
        let json = PlayerCreation::json_string(username).unwrap();
        write_packet_from_json(stream, json);
        let response_packet = read_packet(stream);

        let json_response: Value = serde_json::from_slice(&*response_packet).unwrap();
        match json_response["status"].as_u64() {
            Some(OK_PL_CREAT) => {
                player_token = match json_response["player_token"].as_str() {
                    Some(token) => {
                        // uuid v4 length
                        if token.len() != 36 {
                            continue;
                        }
                        token.to_owned()
                    }
                    None => continue,
                };
            }
            Some(_) | None => continue,
        }
        break;
    }

    player_token
}

fn create_game(stream: &mut TcpStream, player_token: &str) -> String {
    let mut game_token = String::new();
    let json = GameCreation::json_string(player_token).unwrap();
    write_packet_from_json(stream, json);
    let response_packet = read_packet(stream);

    let json_response: Value = serde_json::from_slice(&*response_packet).unwrap();
    match json_response["status"].as_u64() {
        Some(OK_GM_CREAT) => {
            game_token = match json_response["game_token"].as_str() {
                Some(token) => {
                    // uuid v4 length
                    if token.len() != 36 {
                        panic!();
                    }
                    token.to_owned()
                }
                None => panic!(),
            };
        }
        _ => panic!(),
    }

    game_token
}

fn join_game(stream: &mut TcpStream, p_infos: &PlayerInfos) -> Option<String> {
    let json = GameJoining::json_string(&p_infos.player_token, &p_infos.game_token).unwrap();
    write_packet_from_json(stream, json);

    let response_packet = read_packet(stream);
    let json_response: Value = serde_json::from_slice(&*response_packet).unwrap();
    if json_response["status"].as_u64() == Some(OK_GM_JOIN) {
        if let Some(p_vec) = json_response["player_vec"].as_array() {
            for player in p_vec {
                if let Some(username) = player[1].as_str() {
                    if username == p_infos.username {
                        if let Some(p_num) = player[0].as_str() {
                            return Some(p_num.into());
                        }
                        break;
                    }
                } else {
                    break;
                }
            }
        }
    }

    None
}

fn choose_character(stream: &mut TcpStream, p_infos: &PlayerInfos, character_str: &str) -> bool {
    let json =
        ChooseCharacter::json_string(&p_infos.player_token, &p_infos.game_token, character_str)
            .unwrap();
    write_packet_from_json(stream, json);

    let response_packet = read_packet(stream);
    let json_response: Value = serde_json::from_slice(&*response_packet).unwrap();
    if json_response["status"].as_u64() == Some(OK_CHAR_CHOOSING) {
        return true;
    }

    false
}

fn start_game(stream: &mut TcpStream, host_infos: &PlayerInfos) -> Option<(String, String)> {
    let json = GameStarting::json_string(&host_infos.player_token, &host_infos.game_token).unwrap();
    write_packet_from_json(stream, json);

    let response_packet = read_packet(stream);
    let json_response: Value = serde_json::from_slice(&*response_packet).unwrap();
    //println!("start game packet from host {:?}", json_response);
    if json_response["status"].as_u64() == Some(OK_GM_START) {
        if let Some(turn) = json_response["player_turn"].as_str() {
            if let Some(map) = json_response["map"].as_str() {
                return Some((turn.into(), map.into()));
            }
        };
    }

    None
}

#[derive(Serialize, Deserialize)]
struct GameDataResponse {
    status: u64,
    data_type: u64,
    player_num: String,
    player_turn: String,
    map: String,
    // (enemy_number, enemy_remaining_hp)
    enemy: (String, u8),
}

#[derive(Debug)]
struct GameDataFields {
    data_type: u64,
    player_num: String,
    player_turn: String,
    map: String,
    enemy: (String, u8),
}

fn send_game_data(
    stream: &mut TcpStream,
    p_infos: &PlayerInfos,
    game_data: GameDataType,
) -> Option<GameDataFields> {
    let json = match game_data {
        GameDataType::Attack(pos) => {
            GameData::json_string(GM_DATA_ATK, &p_infos.player_token, &p_infos.game_token, pos)
        }
        GameDataType::Movement(pos) => {
            GameData::json_string(GM_DATA_MOV, &p_infos.player_token, &p_infos.game_token, pos)
        }
        GameDataType::Skip => GameData::json_string(
            GM_DATA_SKIP,
            &p_infos.player_token,
            &p_infos.game_token,
            Point(-1, -1),
        ),
    }
    .unwrap();
    write_packet_from_json(stream, json);

    let response_packet = read_packet(stream);
    let gm_data: GameDataResponse = serde_json::from_slice(&*response_packet).unwrap();
    if gm_data.status == OK_GM_DATA {
        return Some(GameDataFields {
            data_type: gm_data.data_type,
            player_num: gm_data.player_num,
            player_turn: gm_data.player_turn,
            map: gm_data.map,
            enemy: gm_data.enemy,
        });
    }

    None
}

fn terminate_connection(stream: &mut TcpStream) -> bool {
    let json = json!({ "request_type": TERM_CON }).to_string();
    write_packet_from_json(stream, json);

    let response_packet = read_packet(stream);
    let json_response: Value = serde_json::from_slice(&*response_packet).unwrap();
    println!(
        "term con packet: {:?} | stream: {:?}",
        json_response, stream
    );
    if json_response["status"].as_u64() == Some(OK_TERM_CON) {
        return true;
    }

    false
}

struct PlayerInfos {
    game_token: String,
    player_token: String,
    username: String,
}
impl PlayerInfos {
    fn new(game_token: String, player_token: String, username: String) -> Self {
        Self {
            game_token,
            player_token,
            username,
        }
    }
}

fn handle_cli_game_action(stream: &mut TcpStream, p_infos: PlayerInfos) {
    let mut input = String::new();
    loop {
        println!("what do you want to do? [mov], [atk], [skip] or [quit]:");
        input.clear();
        stdin().read_line(&mut input).unwrap();
        let gm_type = match &*input.trim().to_lowercase() {
            "mov" => {
                println!("enter coordinate [x,y]:");
                input.clear();
                stdin().read_line(&mut input).unwrap();
                let mut s = input.trim().split(",");
                let x = s.next().unwrap().parse::<i16>().unwrap();
                let y = s.next().unwrap().parse::<i16>().unwrap();
                let point = Point(x, y);
                GameDataType::Movement(point)
            }
            "atk" => {
                println!("enter target coordinate [x,y]:");
                input.clear();
                stdin().read_line(&mut input).unwrap();
                let mut s = input.trim().split(",");
                let x = s.next().unwrap().parse::<i16>().unwrap();
                let y = s.next().unwrap().parse::<i16>().unwrap();
                let point = Point(x, y);
                GameDataType::Attack(point)
            }
            "skip" => GameDataType::Skip,
            "quit" => return,
            _ => continue,
        };

        let gm_data = send_game_data(stream, &p_infos, gm_type.clone()).unwrap();
        match gm_type {
            GameDataType::Movement(_) => println!("{}", gm_data.map),
            GameDataType::Attack(_) => println!(
                "map:\n{}\nenemy number {} has {} hp remaining",
                gm_data.map, gm_data.enemy.0, gm_data.enemy.1
            ),
            GameDataType::Skip => println!("{}\nturn skipped", gm_data.map),
        }

        handle_cli_game_action_reading(stream);
    }
}

fn handle_cli_game_action_reading(stream: &mut TcpStream) {
    let packet = read_packet(stream);
    let gm_data: GameDataResponse = serde_json::from_slice(&*packet).unwrap();
    print!("player {} ", gm_data.player_num);
    match gm_data.data_type {
        GM_DATA_MOV => println!("moved\nmap:\n{}", gm_data.map),
        GM_DATA_ATK => println!("attacked\nmap:\n{}\nplayer {} remaining hp: {}", gm_data.map, gm_data.enemy.0, gm_data.enemy.1),
        GM_DATA_SKIP => println!("skipped his turn"),
        _ => panic!(),
    }
}

fn main() -> anyhow::Result<()> {
    //test_clients();
    let mut stream = TcpStream::connect("127.0.0.1:8000").unwrap();
    loop {
        println!("choose [host] or [player]:");
        let mut input = String::new();
        stdin().read_line(&mut input).unwrap();
        match &*input.trim().to_lowercase() {
            "host" => {
                println!("host player chosen\nchoose username:");
                input.clear();
                stdin().read_line(&mut input).unwrap();
                let username = input.trim();
                let host_player_token = create_player(&mut stream, username);
                println!("player {username} created\nplayer token: {host_player_token}");

                let game_token = create_game(&mut stream, &host_player_token);
                println!("game created\ngame token: {game_token}");
                let p_infos = PlayerInfos::new(game_token, host_player_token, username.into());

                let mut character;
                loop {
                    println!("pick your character [mag], [bar] or [bow]:");
                    input.clear();
                    stdin().read_line(&mut input).unwrap();
                    character = input.trim().to_lowercase();
                    match &*character {
                        "mag" | "bar" | "bow" => break,
                        _ => continue,
                    }
                }

                if choose_character(&mut stream, &p_infos, &character) {
                    println!(
                        "character {character} successfully picked\nwaiting for players to join..."
                    );
                } else {
                    panic!()
                }

                let packet: Value =
                    serde_json::from_str(std::str::from_utf8(&read_packet(&mut stream)).unwrap())
                        .unwrap();

                let p_username = packet["pseudo"].as_str().unwrap();
                println!("player 2 ({p_username}) joined\nwaiting for player 2 to pick a character...");

                //todo: character choosing
                let packet = read_packet(&mut stream);
                println!("player 2 picked his character");

                loop {
                    println!("start game? [y/n]:");
                    input.clear();
                    stdin().read_line(&mut input).unwrap();
                    match &*input.trim().to_lowercase() {
                        "y" | "yes" => break,
                        _ => continue,
                    }
                }

                let (mut turn, map) = start_game(&mut stream, &p_infos).unwrap();
                println!("game started");
                if turn == "1" {
                    println!("you play first\nmap:\n{map}");
                } else {
                    println!("map:\n{map}\nplayer {turn} ({p_username}) is the first to play");
                    handle_cli_game_action_reading(&mut stream);
                }

                handle_cli_game_action(&mut stream, p_infos);
                terminate_connection(&mut stream);
                println!("connection to server closed");
                break;
            }
            "player" => {
                println!("normal player chosen\nchoose username:");
                input.clear();
                stdin().read_line(&mut input).unwrap();
                let username = input.trim().to_owned();
                let player_token = create_player(&mut stream, &username);
                println!("player {username} created\nplayer token: {player_token}\nenter game token to join it:");
                input.clear();
                stdin().read_line(&mut input).unwrap();
                let game_token = input.trim();
                let p_infos = PlayerInfos::new(game_token.into(), player_token, username);

                //todo: players_vec return from join_game
                let player_number = join_game(&mut stream, &p_infos).unwrap();
                println!("game joined\nplayer number: {player_number}");
                let mut character;
                loop {
                    println!("pick your character [mag], [bar] or [bow]:");
                    input.clear();
                    stdin().read_line(&mut input).unwrap();
                    character = input.trim().to_lowercase();
                    match &*character {
                        "mag" | "bar" | "bow" => break,
                        _ => continue,
                    }
                }

                if choose_character(&mut stream, &p_infos, &character) {
                    println!(
                        "character {character} successfully picked\nwaiting for game to start..."
                    );
                } else {
                    panic!()
                }

                let packet = read_packet(&mut stream);
                let json: Value = serde_json::from_slice(&*packet).unwrap();
                let turn = json["player_turn"].as_str().unwrap();
                let map = json["map"].as_str().unwrap();
                println!("game started");

                if turn == player_number {
                    println!("you play first\nmap:\n{map}");
                } else {
                    println!("map:\n{map}\nplayer {turn} is the first to play");
                    handle_cli_game_action_reading(&mut stream);
                }

                handle_cli_game_action(&mut stream, p_infos);
                terminate_connection(&mut stream);
                println!("connection to server closed");
                break;
            }
            _ => continue,
        }
    }

    Ok(())
}

//todo: remove loops to avoid spamming server
