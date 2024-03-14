use crate::{
    choose_character, create_game, create_player, join_game, read_packet, send_game_data,
    start_game, terminate_connection, PlayerInfos,
};
use net_utils::map::{GameDataType, Point};
use serde_json::Value;
use std::net::TcpStream;

pub fn test_clients() {
    let mut stream = TcpStream::connect("127.0.0.1:8000").unwrap();
    let host_player_token = create_player(&mut stream, "coco");
    println!("host player token: {host_player_token}");
    let game_token = create_game(&mut stream, &host_player_token);
    println!("game token: {game_token}");
    let host_infos = PlayerInfos::new(game_token, host_player_token, "coco".into());

    let g_token = host_infos.game_token.clone();
    let handle = std::thread::spawn(move || {
        let mut stream = TcpStream::connect("127.0.0.1:8000").unwrap();
        let p_token = create_player(&mut stream, "bob");
        let bob_infos = PlayerInfos::new(g_token, p_token, "bob".into());
        println!("bob token: {}", bob_infos.player_token);
        let player_number = match join_game(&mut stream, &bob_infos) {
            Some(p_num) => {
                println!("bob joined | player number: {p_num}");
                p_num
            }
            None => panic!(),
        };

        if choose_character(&mut stream, &bob_infos, "bar") {
            println!("bob chose barbarian");
        }

        let packet = read_packet(&mut stream);
        let json: Value = serde_json::from_slice(&*packet).unwrap();
        let turn = match json["player_turn"].as_str() {
            Some(t) => t,
            None => panic!(),
        };
        //println!(
        //    "game starting packet from bob: {:?}",
        //    json
        //);

        let gm_data = if turn == player_number {
            match send_game_data(&mut stream, &bob_infos, GameDataType::Movement(Point(7, 4))) {
                Some(gm_data) => {
                    println!("bob sent data to the server: {:?}", gm_data);

                    let packet = read_packet(&mut stream);
                    println!("host game data received from bob: {:?}", packet);
                    //println!("phost game data packet from bob: {:?}", std::str::from_utf8(&packet));

                    gm_data
                }
                None => panic!(),
            }
        } else {
            let packet = read_packet(&mut stream);
            println!("host game data received from bob: {:?}", packet);

            match send_game_data(&mut stream, &bob_infos, GameDataType::Movement(Point(7, 4))) {
                Some(gm_data) => {
                    println!("bob sent data to the server: {:?}", gm_data);
                    gm_data
                }
                None => panic!(),
            }
        };

        if terminate_connection(&mut stream) {
            println!("player bob terminated connection");
        }
    });

    if choose_character(&mut stream, &host_infos, "mag") {
        println!("phost chose magician");
    }

    // reading bob joining packets
    let packet = read_packet(&mut stream);
    //println!("bob joining packet from phost: {:?}", std::str::from_utf8(&packet));

    let packet = read_packet(&mut stream);
    //println!("bob character choosing from phost: {:?}", std::str::from_utf8(&packet));

    let (turn, _map) = match start_game(&mut stream, &host_infos) {
        Some(t) => {
            println!("game started");
            t
        }
        None => panic!(),
    };

    //todo: send game data
    if turn == "1" {
        let gm_data = send_game_data(
            &mut stream,
            &host_infos,
            GameDataType::Movement(Point(1, 0)),
        ).unwrap();
        println!("host player sent data to the server: {:?}", gm_data);
        let packet = read_packet(&mut stream);
        println!(
            "host player received game data of bob from the server: {:?}",
            std::str::from_utf8(&packet)
        );
    } else {
        let packet = read_packet(&mut stream);
        println!(
            "host player received game data of bob from the server: {:?}",
            std::str::from_utf8(&packet)
        );
        let gm_data = send_game_data(
            &mut stream,
            &host_infos,
            GameDataType::Movement(Point(1, 0)),
        ).unwrap();
        println!("host player sent data to the server: {:?}", gm_data);
    }

    handle.join().unwrap();
    if terminate_connection(&mut stream) {
        println!("player host terminated connection");
    }
}
