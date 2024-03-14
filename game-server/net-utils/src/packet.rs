pub mod request_codes {
    pub const TERM_CON: u64 = 10;
    pub const PL_CREAT: u64 = 11;
    pub const GM_CREAT: u64 = 12;
    pub const GM_JOIN: u64 = 13;
    pub const CHAR_CHOOSING: u64 = 14;
    pub const GM_START: u64 = 15;
    pub const GM_DATA: u64 = 16;
}

pub mod status_codes {
    // terminate tcp connection
    pub const OK_TERM_CON: u64 = 20;
    // player created
    pub const OK_PL_CREAT: u64 = 21;
    // game created
    pub const OK_GM_CREAT: u64 = 22;
    // game joining
    pub const OK_GM_JOIN: u64 = 23;
    // character chosen
    pub const OK_CHAR_CHOOSING: u64 = 24;
    // game started
    pub const OK_GM_START: u64 = 25;
    // game data
    pub const OK_GM_DATA: u64 = 26;

    pub const ERR_INTERNAL_SERV: u64 = 30;
    // malformed request
    pub const ERR_MAL_REQ: u64 = 31;
    // invalid pseudo
    pub const ERR_INV_PSEUD: u64 = 32;
    // invalid player token
    pub const ERR_INV_PL_TOK: u64 = 33;
    // invalid game token
    pub const ERR_INV_GM_TOK: u64 = 34;
    // game already started
    pub const ERR_GM_AL_START: u64 = 35;
    // game full (4 players)
    pub const ERR_GM_FULL: u64 = 36;
    // game not joined (can't send game data)
    pub const ERR_GM_NOT_JOIN: u64 = 37;
    // game not full (can't start)
    pub const ERR_GM_NOT_FULL: u64 = 38;
    // game not started (can't send game data)
    pub const ERR_GM_NOT_START: u64 = 39;
}

pub mod game_data_code {
    // player movement
    pub const GM_DATA_MOV: u64 = 50;
    // player attack
    pub const GM_DATA_ATK: u64 = 51;
    // skip turn
    pub const GM_DATA_SKIP: u64 = 52;
}
