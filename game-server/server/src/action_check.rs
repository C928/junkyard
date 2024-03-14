use crate::response::GamePlayerInfos;
use net_utils::map::Point;
use serde::{Deserialize, Serialize};
use std::cmp::max;
use std::str::FromStr;

#[derive(Deserialize, Serialize)]
pub struct PlayerInfos {
    pseudo: String,
    hosting: u8,
}

fn astar(_map: &mut String, _start: Point, _dest: &Point) -> i16 {
    //todo
    return 1;
}

fn map_string_to_vec(str_map: &mut String) -> Vec<Vec<char>> {
    str_map.lines().map(|line| line.chars().collect()).collect()
}

fn map_vec_to_string(map_vec: Vec<Vec<char>>) -> String {
    map_vec
        .iter()
        .map(|line| line.iter().collect::<String>())
        .collect::<Vec<String>>()
        .join("\n")
}

pub fn reach_destination(
    map: &mut String,
    player_number: String,
    dest: Point,
    character_ms: u8,
) -> bool {
    let mut map_vec = map_string_to_vec(map);
    let height = map_vec.len();
    let width = map_vec[0].len();
    if dest.0 > (width - 1) as i16 || dest.1 > (height - 1) as i16 {
        return false;
    }

    let player_number = char::from_str(&*player_number).unwrap();
    for y in 0..height {
        for x in 0..width {
            if map_vec[y][x] == player_number {
                let start = Point(x as i16, y as i16);
                let distance = astar(map, start, &dest);
                if distance > character_ms as i16 {
                    return false;
                }

                return if distance != -1 {
                    map_vec[y][x] = '0';
                    map_vec[dest.1 as usize][dest.0 as usize] = player_number;
                    map.clear();
                    let map_str = map_vec_to_string(map_vec);
                    map.push_str(&*map_str);
                    true
                } else {
                    false
                };
            }
        }
    }

    false
}

// Some((enemy_player_number, enemy_remaining_hp)
pub fn player_attack(
    map: &mut String,
    player_num: String,
    enemy_vec: Vec<GamePlayerInfos>,
    target: Point,
    player_stats: (u8, u8),
) -> Option<(String, u8)> {
    let player_num = char::from_str(&*player_num).unwrap();
    let mut map_vec = map_string_to_vec(map);
    let height = map_vec.len();
    let width = map_vec[0].len();
    println!("h: {height}, w: {width}");

    if target.0 > (height - 1) as i16 || target.1 > (width - 1) as i16 {
        return None;
    }

    let mut ok = false;
    for y in 0..height {
        for x in 0..width {
            if map_vec[y][x] == player_num {
                if max((x as i16 - target.0).abs(), (y as i16 - target.1).abs())
                    > player_stats.1 as i16
                {
                    return None;
                }
                ok = true;
            }
        }
    }
    if !ok {
        return None;
    }

    let enemy = map_vec[target.1 as usize][target.0 as usize].to_string();
    for enemy_infos in enemy_vec {
        if enemy_infos.player_num == &*enemy {
            // enemy hp - player atk
            let enemy_new_hp = enemy_infos.stats.1 - player_stats.1;
            if enemy_new_hp == 0 {
                map_vec[target.1 as usize][target.0 as usize] = '0';
            }
            map.clear();
            map.push_str(&*map_vec_to_string(map_vec));
            //new enemy hp
            return Some((enemy.into(), enemy_new_hp));
        }
    }

    None
}
