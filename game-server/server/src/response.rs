use net_utils::packet::status_codes::*;
use serde::{Deserialize, Serialize};

pub mod packet_sizes {
    // default: only status code
    pub const DEF: u16 = 13;

    // ok responses
    // terminate connection
    pub const TERM_CON_SIZE: u16 = DEF;
    // player creation
    pub const PL_CREAT_SIZE: u16 = 67;
    // game creation
    pub const GM_CREAT_SIZE: u16 = 65;

    // err responses
    // malformed request
    pub const MAL_REQ_SIZE: u16 = DEF;
    // invalid pseudo
    pub const INV_PSEUD_SIZE: u16 = DEF;
    // invalid player token
    pub const INV_PL_TOK_SIZE: u16 = DEF;
    // invalid game token
    pub const INV_GM_TOK_SIZE: u16 = DEF;
    // game already started
    pub const GM_AL_START_SIZE: u16 = DEF;
    // game full (4 players)
    pub const GM_FULL_SIZE: u16 = DEF;
    // game not full (can't start)
    pub const GM_NOT_FULL_SIZE: u16 = DEF;
    // game not joined (can't send game data)
    pub const GM_NOT_JOIN_SIZE: u16 = DEF;
    // game not started (can't send game data)
    pub const GM_NOT_START_SIZE: u16 = DEF;
}

#[derive(Deserialize, Serialize)]
pub struct GamePlayerInfos {
    pub player_num: String,
    pub character: String,
    // atk, hp, ms, rng
    pub stats: (u8, u8, u8, u8),
}
impl GamePlayerInfos {
    pub fn json_string(
        player_num: String,
        character: String,
        stats: (u8, u8, u8, u8),
    ) -> serde_json::Result<String> {
        serde_json::to_string(&Self {
            player_num,
            character,
            stats,
        })
    }
}

////

#[derive(Serialize, Deserialize)]
pub struct PlayerCreation<'a> {
    status: u64,
    player_token: &'a str,
}
impl<'a> PlayerCreation<'a> {
    pub fn json_string(player_token: &'a str) -> serde_json::Result<String> {
        serde_json::to_string(&Self {
            status: OK_PL_CREAT,
            player_token,
        })
    }
}

#[derive(Serialize, Deserialize)]
pub struct GameCreation<'a> {
    status: u64,
    game_token: &'a str,
}
impl<'a> GameCreation<'a> {
    pub fn json_string(game_token: &'a str) -> serde_json::Result<String> {
        serde_json::to_string(&Self {
            status: OK_GM_CREAT,
            game_token,
        })
    }
}

#[derive(Serialize, Deserialize)]
pub struct GameJoining<'a> {
    status: u64,
    // pseudo of newly joined player
    pseudo: &'a str,
    // player currently in the game Vec<[player_number, pseudo, character, is_host]>
    player_vec: Vec<[String; 4]>,
}
impl<'a> GameJoining<'a> {
    pub fn json_string(
        pseudo: &'a str,
        player_vec: Vec<[String; 4]>,
    ) -> serde_json::Result<String> {
        serde_json::to_string(&Self {
            status: OK_GM_JOIN,
            pseudo,
            player_vec,
        })
    }
}

#[derive(Serialize, Deserialize)]
pub struct CharacterChoosing<'a> {
    status: u64,
    pseudo: &'a str,
    character: &'a str,
}
impl<'a> CharacterChoosing<'a> {
    pub fn json_string(pseudo: &'a str, character: &'a str) -> serde_json::Result<String> {
        serde_json::to_string(&Self {
            status: OK_CHAR_CHOOSING,
            pseudo,
            character,
        })
    }
}

#[derive(Serialize, Deserialize)]
pub struct GameStarting {
    status: u64,
    player_turn: String,
    map: String,
}
impl GameStarting {
    pub fn json_string(player_turn: String, map: String) -> serde_json::Result<String> {
        serde_json::to_string(&Self {
            status: OK_GM_START,
            player_turn,
            map,
        })
    }
}

#[derive(Serialize, Deserialize)]
pub struct GameData {
    status: u64,
    data_type: u64,
    player_num: String,
    player_turn: String,
    map: String,
    // (enemy_number, enemy_remaining_hp)
    enemy: (String, u8),
}
impl GameData {
    pub fn json_string(
        data_type: u64,
        player_turn: String,
        player_num: String,
        map: String,
        enemy: (String, u8),
    ) -> serde_json::Result<String> {
        serde_json::to_string(&Self {
            status: OK_GM_DATA,
            data_type,
            player_num,
            player_turn,
            map,
            enemy,
        })
    }
}
