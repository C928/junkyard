use net_utils::character::CharacterClass;
use net_utils::map::Point;
use net_utils::packet::request_codes::*;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct PlayerCreation<'a> {
    request_type: u64,
    pseudo: &'a str,
}
impl<'a> PlayerCreation<'a> {
    pub fn json_string(pseudo: &'a str) -> serde_json::Result<String> {
        serde_json::to_string(&Self {
            request_type: PL_CREAT,
            pseudo,
        })
    }
}

#[derive(Serialize, Deserialize)]
pub struct GameCreation<'a> {
    request_type: u64,
    player_token: &'a str,
}
impl<'a> GameCreation<'a> {
    pub fn json_string(player_token: &'a str) -> serde_json::Result<String> {
        serde_json::to_string(&Self {
            request_type: GM_CREAT,
            player_token,
        })
    }
}

#[derive(Serialize, Deserialize)]
pub struct GameJoining<'a> {
    request_type: u64,
    player_token: &'a str,
    game_token: &'a str,
}
impl<'a> GameJoining<'a> {
    pub fn json_string(player_token: &'a str, game_token: &'a str) -> serde_json::Result<String> {
        serde_json::to_string(&Self {
            request_type: GM_JOIN,
            player_token,
            game_token,
        })
    }
}

#[derive(Serialize, Deserialize)]
pub struct GameStarting<'a> {
    request_type: u64,
    player_token: &'a str,
    game_token: &'a str,
}
impl<'a> GameStarting<'a> {
    pub fn json_string(player_token: &'a str, game_token: &'a str) -> serde_json::Result<String> {
        serde_json::to_string(&Self {
            request_type: GM_START,
            player_token,
            game_token,
        })
    }
}

#[derive(Serialize, Deserialize)]
pub struct ChooseCharacter<'a> {
    request_type: u64,
    player_token: &'a str,
    game_token: &'a str,
    character: &'a str,
}
impl<'a> ChooseCharacter<'a> {
    pub fn json_string(
        player_token: &'a str,
        game_token: &'a str,
        character: &'a str,
    ) -> serde_json::Result<String> {
        serde_json::to_string(&Self {
            request_type: CHAR_CHOOSING,
            player_token,
            game_token,
            character,
        })
    }
}

#[derive(Serialize, Deserialize)]
pub struct GameData<'a> {
    request_type: u64,
    gm_code: u64,
    player_token: &'a str,
    game_token: &'a str,
    target: Point,
}
impl<'a> GameData<'a> {
    pub fn json_string(
        gm_code: u64,
        player_token: &'a str,
        game_token: &'a str,
        target: Point,
    ) -> serde_json::Result<String> {
        serde_json::to_string(&Self {
            request_type: GM_DATA,
            gm_code,
            player_token,
            game_token,
            target,
        })
    }
}
