use chrono::{Datelike, Duration, Local, NaiveDate};
use rand::Rng;
use std::cmp::min;
use std::env::{current_dir, set_current_dir};
use std::fs;
use std::fs::File;
use std::io::{Read, Seek, SeekFrom, Write};
use std::ops::AddAssign;
use std::os::unix::fs::OpenOptionsExt;
use std::path::{Path, PathBuf};

use crate::utils::is_cal_file;

// (12*24*100)
const MAX_EVENT: usize = 28800;
const MAX_EVENT_STR: &str = "1 event every 5 minutes";

struct Event {
    e_id: u16,
    e_type: char,
    e_name: String,
    e_desc: String,
    e_date: (i32, u32, u32),
    e_time: (u32, u32),
}

//todo first: derive default to remove new_empty_cal() and maybe is_empty()
//todo first: refactor function name to remove redundant 'cal'
pub struct Calendar {
    filename: String,
    fd: Option<File>,
    init_dates: Option<(i32, u32, u32, i32, u32, u32)>,
    events_vec: Vec<Event>,
}

impl Calendar {
    pub fn new(mut filename: String, input_date: Option<String>) -> Option<Self> {
        let file;
        let fd;
        let init_dates;
        if filename == "" {
            eprintln!("Error: filename cannot be empty");
            return None;
        }

        //todo first
        if Path::new(".backup").exists() {

        }
        /*
        let dir_files = match fs::read_dir("./") {
            Ok(f) => f,
            Err(e) => {
                eprintln!("Error: Directory files could not be listed");
                return None;
            }
        };
         */

        if Path::new(&filename).exists() {
            file = fs::OpenOptions::new()
                .read(true)
                .write(true)
                .open(&filename);
            fd = match file {
                Ok(f) => Some(f),
                Err(_) => {
                    eprintln!("Error: Opening '{}'", filename);
                    return None;
                }
            };
            init_dates = None;
        } else {
            if input_date == None {
                eprintln!("Error: Specify the init date with '-i' in this format: YYYY-MM-DD");
                return None;
            }

            if !is_cal_file(&filename) {
                filename = format!("{}.cal", filename);
            }

            file = fs::OpenOptions::new()
                .create_new(true)
                .read(true)
                .write(true)
                .mode(0o644)
                .open(&filename);

            fd = match file {
                Ok(f) => Some(f),
                Err(_) => {
                    eprintln!("Error: Creating file '{}'", filename);
                    return None;
                }
            };

            let date = match Calendar::parse_input_date(input_date.unwrap()) {
                Some(ret) => ret,
                None => {
                    eprintln!("Error: Parsing init date");
                    return None;
                }
            };

            let local = Local::today();
            let d0 = NaiveDate::from_ymd(local.year(), local.month(), local.day());
            let d1 = NaiveDate::from_ymd(date.0, date.1, date.2);
            if Duration::from(d0 - d1).num_days() > 99 {
                eprintln!("Error: Your 100 day challenge is already finished");
                return None;
            }

            let d2 = d1 + Duration::days(99);
            init_dates = Some((
                d1.year(),
                d1.month(),
                d1.day(),
                d2.year(),
                d2.month(),
                d2.day(),
            ));

            //todo first: encrypt before writing to file
            /*
            let cal_str = format!(
                "{}{}-{}-{}-{}-{}-{}{}\n",
                '(', d.0, d.1, d.2, d.3, d.4, d.5, ')'
            );
             */

            let d = init_dates.unwrap();
            match write!(
                fd.as_ref().unwrap(),
                "{}{}-{}-{}-{}-{}-{}{}\n",
                '(',
                d.0,
                d.1,
                d.2,
                d.3,
                d.4,
                d.5,
                ')'
            ) {
                Ok(_) => println!("[+] '{}' created", filename),
                Err(_) => {
                    eprintln!("Error: Writing init dates to '{}'", filename);
                    return None;
                }
            };
        }

        Some(Self {
            filename,
            fd,
            init_dates,
            events_vec: Vec::new(),
        })
    }

    pub fn new_empty_cal() -> Self {
        Self {
            filename: "".to_string(),
            fd: None,
            init_dates: None,
            events_vec: vec![]
        }
    }

    pub fn is_empty(&self) -> bool {
        return match &*self.filename {
            "" => true,
            _ => false,
        }
    }

    pub fn get_cal_contents(&mut self, mut file_contents: &mut String) -> u8 {
        file_contents.clear();
        if Path::new(&self.filename).exists() {
            let mut file = self.fd.as_ref().unwrap();
            file.rewind().unwrap();
            if let Err(_) = file.read_to_string(&mut file_contents) {
                eprintln!("Error: '{}' could not be read", self.filename);
                return 1;
            }
        } else {
            eprintln!("Error: '{}' does not exist", self.filename);
            return 1;
        }

        0
    }

    pub fn parse_calendar(&mut self, file_contents: &String) -> u8 {
        if !file_contents.is_empty() {
            let mut s = file_contents.split(")\n");
            let init_dates = match s.next() {
                Some(ret) => ret[1..].split("-"),
                None => {
                    eprintln!("Error: Parsing '{}'", self.filename);
                    return 1;
                }
            };

            if self.init_dates == None {
                match Calendar::parse_init_dates(init_dates) {
                    Some(d) => self.init_dates = Some(d),
                    None => {
                        eprintln!("Error: Parsing init dates from '{}'", self.filename);
                        return 1;
                    }
                }
            }

            let contents = s.next().unwrap();
            if contents.is_empty() {
                return 0;
            }

            let mut events: Vec<&str> = contents
                .split(";\n")
                .filter(|&x| !x.eq("") && !x.contains("\n")) //todo: !contains("\n")-> in case of new line, encode it
                .collect();

            for e in events {
                let fields: Vec<&str> = e.split(',').collect();
                let mut f_iter = fields.iter();

                let mut id = match f_iter.next() {
                    Some(ret) => ret.split("ᕕ( ᐛ )ᕗ"), //todo
                    None => {
                        eprintln!("Error: Parsing event id from '{}'", self.filename);
                        return 1;
                    }
                };

                let e_id = match Calendar::parse_u16(&mut id) {
                    Some(ret) => ret,
                    None => {
                        eprintln!("Error: Parsing event id from '{}'", self.filename);
                        return 1;
                    }
                };

                let date_time = match f_iter.next() {
                    Some(ret) => ret.split("-"),
                    None => {
                        eprintln!("Error: Parsing event date/time from '{}'", self.filename);
                        return 1;
                    }
                };

                let d = match Calendar::parse_date_time(date_time) {
                    Some(ret) => ret,
                    None => {
                        eprintln!("Error: Parsing event date/time from '{}'", self.filename);
                        return 1;
                    }
                };

                let e_type = match f_iter.next() {
                    Some(ret) => match ret.chars().next() {
                        Some(t) => t,
                        None => {
                            eprintln!("Error: Parsing event type from '{}'", self.filename);
                            return 1;
                        }
                    },
                    None => {
                        eprintln!("Error: Parsing event type from '{}'", self.filename);
                        return 1;
                    }
                };

                let e_name = match f_iter.next() {
                    Some(ret) => ret.to_string(),
                    None => {
                        eprintln!("Error: Parsing event name from '{}'", self.filename);
                        return 1;
                    }
                };

                let e_desc = match f_iter.next() {
                    Some(ret) => ret.to_string(),
                    None => {
                        eprintln!("Error: Parsing event description from '{}'", self.filename);
                        return 1;
                    }
                };

                let event = Event {
                    e_id,
                    e_type,
                    e_name,
                    e_desc,
                    e_date: (d.0, d.1, d.2),
                    e_time: (d.3, d.4),
                };

                self.events_vec.push(event);
            }
        } else {
            eprintln!("Error: Calendar does not contain anything");
            return 1;
        }

        0
    }

    pub fn display_events(&mut self, display_type: u8) -> u8 {
        let print_evt = |e: &Event| {
            println!(
                "{} {} {}-{}-{} {}:{} {}",
                e.e_id,
                e.e_name,
                e.e_date.0,
                e.e_date.1,
                e.e_date.2,
                e.e_time.0,
                e.e_time.1,
                e.e_desc
            );
        };

        let mut tmp_vec: Vec<&Event> = vec![];
        match display_type {
            0 | 1 => {
                // display by date
                for e in &self.events_vec {
                    tmp_vec.push(e);
                }

                let evts_cnt = tmp_vec.len();
                for i in 0..evts_cnt {
                    for j in 0..evts_cnt - i - 1 {
                        let a = (tmp_vec[j].e_date, tmp_vec[j].e_time);
                        let b = (tmp_vec[j + 1].e_date, tmp_vec[j + 1].e_time);
                        let date_a =
                            NaiveDate::from_ymd(a.0 .0, a.0 .1, a.0 .2).and_hms(a.1 .0, a.1 .1, 00);
                        let date_b =
                            NaiveDate::from_ymd(b.0 .0, b.0 .1, b.0 .2).and_hms(b.1 .0, b.1 .1, 00);

                        if date_a < date_b {
                            tmp_vec.swap(j, j + 1);
                        }
                    }
                }

                if display_type == 1 {
                    for e in tmp_vec {
                        print_evt(e);
                    }
                } else {
                    for i in (0..tmp_vec.len()).rev() {
                        print_evt(tmp_vec[i]);
                    }
                }
            }
            2 | 3 => {
                // display by name
                for e in &self.events_vec {
                    tmp_vec.push(e);
                }

                let events_cnt = tmp_vec.len();
                for i in 0..events_cnt {
                    for j in 0..events_cnt - i - 1 {
                        let names_len: (usize, usize) =
                            (tmp_vec[j].e_name.len(), tmp_vec[j + 1].e_name.len());
                        let mut res: u8 = 0;
                        for x in 0..min(names_len.0, names_len.1) {
                            let a = tmp_vec[j].e_name.as_bytes()[x] as char;
                            let b = tmp_vec[j + 1].e_name.as_bytes()[x] as char;
                            if a < b {
                                res = 1;
                                break;
                            } else if a > b {
                                res = 2;
                                break;
                            }
                        }
                        if res == 0 && names_len.0 != names_len.1 {
                            res = if names_len.0 < names_len.1 { 1 } else { 2 };
                        }

                        if res == 2 {
                            tmp_vec.swap(j, j + 1);
                        }
                    }
                }

                if display_type == 3 {
                    for e in tmp_vec {
                        print_evt(e);
                    }
                } else {
                    for i in (0..tmp_vec.len()).rev() {
                        print_evt(tmp_vec[i]);
                    }
                }
            }
            4 | 5 => {
                // display by date of addition
                if display_type == 5 {
                    for i in (0..self.events_vec.len()).rev() {
                        print_evt(&self.events_vec[i]);
                    }
                } else {
                    for e in &self.events_vec {
                        print_evt(e);
                    }
                }
            }
            _ => {
                eprintln!(
                    "Error: Display number must be between 0 and 5. Got '{}'.",
                    display_type
                );
                return 1;
            }
        }

        0
    }

    pub fn add_event(
        &mut self,
        e_type: char,
        e_name: String,
        desc: Option<String>,
        e_date: (i32, u32, u32),
        e_time: (u32, u32),
        output: bool,
    ) -> u8 {
        // if self.events_vec.len() as u16 == u16::MAX {
        if self.events_vec.len() == MAX_EVENT {
            eprintln!(
                "Error: Event could not be added because there already is '{}' events.\n\
                It corresponds to {}",
                MAX_EVENT, MAX_EVENT_STR,
            );
            return 1;
        }

        if "OoXx".contains(e_type) {
            //todo: with or without description
            if e_name.len() > 32 {
                eprintln!("Error: Event name must have <= 32 characters");
                return 1;
            }

            let e_desc = match desc {
                Some(d) => {
                    if d.len() > 256 {
                        eprintln!("Error: Event description must have <= 256 characters");
                        return 1;
                    }
                    // d.replace("\n", " ").trim() //todo: encode "\n"
                    d
                }
                None => "none".to_owned(),
            };

            let init_d = self.init_dates.unwrap();
            let d1 = NaiveDate::from_ymd(init_d.0, init_d.1, init_d.2);
            let d2 = NaiveDate::from_ymd(init_d.3, init_d.4, init_d.5);
            let input_date = NaiveDate::from_ymd(e_date.0, e_date.1, e_date.2);

            if input_date < d1 || input_date > d2 {
                eprintln!("Error: Adding event outside the 100 days scope");
                return 1;
            }

            if e_time.0 > 23 || e_time.1 > 59 {
                eprintln!("Error: Event time"); //todo
                return 1;
            }

            let mut e_id: u16 = 0;
            let mut ok = false;
            while !ok {
                e_id = rand::thread_rng().gen();
                ok = true;
                for e in &self.events_vec {
                    if e.e_id == e_id {
                        ok = false;
                        break;
                    }
                }
            }

            /*
            let fmt = format!(
                "{},{}-{}-{},{},{};",
                e_id, input_date, e_time.0, e_time.1, e_name, e_desc,
            );

            let str = match e_type {
                'O' => format!("{},O,{};", fmt_date, e_name),
                'o' => format!("{},o;", fmt_date),
                'X' => format!("{},X,{};", fmt_date, e_name),
                'x' => format!("{},x;", fmt_date),
                _ => return Err("Error: Event parsing failed"),
            };
            */

            let mut fd = self.fd.as_ref().unwrap();
            fd.seek(SeekFrom::End(0x00));
            match writeln!(
                fd,
                "{},{}-{}-{},{},{},{};",
                e_id, input_date, e_time.0, e_time.1, e_type, e_name, e_desc,
            ) {
                Ok(_) => {
                    if output {
                        println!("[+] Event '{}' written to '{}'", e_name, self.filename);
                    }

                    let event = Event {
                        e_id,
                        e_type,
                        e_name,
                        e_desc,
                        e_date: (e_date.0, e_date.1, e_date.2),
                        e_time: (e_time.0, e_time.1),
                    };

                    self.events_vec.push(event);
                }
                Err(_) => {
                    eprintln!("Error: Writing event to '{}'", self.filename);
                    return 1;
                }
            };
        } else {
            eprintln!("Error: Event type must be 'O','o', 'X' or 'x'");
            return 1;
        }

        0
    }

    pub fn add_repeating_event(
        &mut self,
        e_days: String,
        e_type: char,
        e_name: String,
        desc: Option<String>,
        e_dt: (i32, u32, u32, u32, u32),
    ) -> u8 {
        if e_days.len() > 20 {
            eprintln!("Error: Parsing days of repeating pattern");
            return 1;
        }

        let d_array = ["su", "mo", "tu", "we", "th", "fr", "sa"];
        let days = e_days.split("-");
        let mut d_map = vec![];
        for d in days {
            if d == "*" {
                //d_map[0] = "*";
                d_map.clear();
                d_map.push("*");
                break;
            } else if d_array.contains(&d) {
                d_map.push(d);
            } else {
                eprintln!("Error: Parsing days of repeating pattern");
                return 1;
            }
        }

        let init_d = &self.init_dates.unwrap();
        let mut s_date = NaiveDate::from_ymd(e_dt.0, e_dt.1, e_dt.2);
        let init_date = NaiveDate::from_ymd(init_d.3, init_d.4, init_d.5);
        let dif = (init_date - s_date).num_days();

        if dif < 0 || dif > 99 {
            eprintln!(
                "Error: Starting date for repeating event does not fit inside the 100 day scope"
            );
            return 1;
        }

        //todo: clone()...
        for _i in 0..dif + 1 {
            let mut day = s_date.weekday().to_string().to_lowercase();
            if d_map[0] == "*" || d_map.contains(&&day[0..2]) {
                if self.add_event(
                    e_type,
                    e_name.clone(),
                    desc.clone(),
                    (s_date.year(), s_date.month(), s_date.day()),
                    (e_dt.3, e_dt.4),
                    false,
                ) == 1
                {
                    return 1;
                }
            }

            s_date.add_assign(Duration::days(1));
        }
        println!("[+] Repeating event written to '{}'", self.filename);

        0
    }

    pub fn modify_event(&mut self, e_id: u16, e_name: String, desc: Option<String>) -> u8 {
        let mut e_str = String::new();
        if self.remove_event(e_id, &mut e_str, false) == 1 {
            return 1;
        }

        let e_date_time = match e_str.split(',').nth(1) {
            Some(ret) => ret,
            None => {
                eprintln!("Error: Parsing event");
                return 1;
            }
        };

        let (e_date, e_time) = match Calendar::parse_date_time(e_date_time.split("-")) {
            Some(ret) => ((ret.0, ret.1, ret.2), (ret.3, ret.4)),
            None => {
                eprintln!("Error: Parsing event date/time");
                return 1;
            }
        };

        let e_desc = match desc {
            Some(d) => d,
            None => "none".to_owned(),
        };

        match writeln!(
            self.fd.as_ref().unwrap(),
            "{},{}-{}-{}-{}-{},{},{},{};",
            e_id,
            e_date.0,
            e_date.1,
            e_date.2,
            e_time.0,
            e_time.1,
            'x',
            e_name,
            e_desc,
        ) {
            Ok(_) => {
                println!(
                    "[+] Event '{}' successfully modified and written to '{}'",
                    e_id, self.filename
                );
                let event = Event {
                    e_id,
                    e_type: 'x',
                    e_name,
                    e_desc,
                    e_date,
                    e_time,
                };

                self.events_vec.push(event);
            }
            Err(_) => {
                eprintln!("Error: Writing event to '{}'", self.filename);
                return 1;
            }
        }

        0
    }

    //todo: allow removing multiple event (3214 or 234,23421,53 or *)
    pub fn remove_event(&mut self, e_id: u16, removed_evt: &mut String, output: bool) -> u8 {
        let mut contains = false;
        for e in &self.events_vec {
            if e.e_id == e_id {
                contains = true;
                break;
            }
        }

        if contains {
            if Path::new(&self.filename).exists() {
                let mut file = self.fd.as_ref().unwrap();
                file.rewind().unwrap();

                let mut file_contents = String::new();
                if let Err(_) = file.read_to_string(&mut file_contents) {
                    eprintln!("Error: '{}' could not be read", self.filename);
                    return 1;
                }

                /* todo: use get_cal_contents()
                match self.get_cal_contents(&mut file_contents) {
                    Some(ret) => ret,
                    None => return 1,
                };
                 */

                file_contents = match file_contents.split_once("\n") {
                    Some(ret) => {
                        file.set_len(0);
                        file.rewind().unwrap();
                        if let Err(_) = writeln!(file, "{}", ret.0) {
                            eprintln!("Error: Writing '{}'", self.filename);
                            return 1;
                        }
                        ret.1.to_owned()
                    }
                    None => {
                        eprintln!("Error: Parsing file");
                        return 1;
                    }
                };

                let events = file_contents.split(";\n").filter(|&x| !x.eq(""));
                for (index, e) in events.enumerate() {
                    match e.find(&e_id.to_string()) {
                        Some(0) => {
                            if removed_evt.is_empty() {
                                removed_evt.push_str(e);
                            }
                            self.events_vec.remove(index);
                        }
                        _ => {
                            if let Err(_) = writeln!(file, "{};", e) {
                                eprintln!("Error: Writing '{}'", self.filename);
                                return 1;
                            }
                        }
                    }
                }
                if output {
                    println!("[+] Event {} removed from '{}'", e_id, self.filename);
                }
            } else {
                eprintln!("Error: '{}' not found", self.filename);
                return 1;
            }
        } else {
            eprintln!(
                "Error: '{}' does not contains an event with id '{}'",
                self.filename, e_id
            );
            return 1;
        }

        0
    }

    /* todo
    fn encrypt_contents(&self) -> u8 {

        0
    }

    fn decrypt_contents(&self, ) -> u8 {

        0
    }
    */

    fn parse_i32(s: &mut core::str::Split<&str>) -> Option<i32> {
        match s.next() {
            Some(s1) => match s1.parse() {
                Ok(s2) => Some(s2),
                Err(_) => None,
            },
            None => None,
        }
    }

    fn parse_u32(s: &mut core::str::Split<&str>) -> Option<u32> {
        match s.next() {
            Some(s1) => match s1.parse() {
                Ok(s2) => Some(s2),
                Err(_) => None,
            },
            None => None,
        }
    }

    fn parse_u16(s: &mut core::str::Split<&str>) -> Option<u16> {
        match s.next() {
            Some(s1) => match s1.parse() {
                Ok(s2) => Some(s2),
                Err(_) => None,
            },
            None => None,
        }
    }

    fn parse_input_date(str: String) -> Option<(i32, u32, u32)> {
        let mut str_split = str.split("-");
        let i32_val = match Calendar::parse_i32(&mut str_split) {
            Some(ret) => ret,
            None => return None,
        };

        let mut u32_vec: Vec<u32> = Vec::new();
        for _i in 0..2 {
            if let Some(val) = Calendar::parse_u32(&mut str_split) {
                u32_vec.push(val);
            } else {
                return None;
            }
        }

        Some((i32_val, u32_vec[0], u32_vec[1]))
    }

    fn parse_init_dates(
        mut str_split: core::str::Split<&str>,
    ) -> Option<(i32, u32, u32, i32, u32, u32)> {
        let mut i32_vec: Vec<i32> = Vec::new();
        let mut u32_vec: Vec<u32> = Vec::new();
        for n in 0..6 {
            if n == 0 || n == 3 {
                if let Some(val) = Calendar::parse_i32(&mut str_split) {
                    i32_vec.push(val);
                } else {
                    return None;
                }
            } else {
                if let Some(val) = Calendar::parse_u32(&mut str_split) {
                    u32_vec.push(val);
                } else {
                    return None;
                }
            }
        }

        Some((
            i32_vec[0], u32_vec[0], u32_vec[1], i32_vec[1], u32_vec[2], u32_vec[3],
        ))
    }

    pub fn parse_date_time(
        mut str_split: core::str::Split<&str>,
    ) -> Option<(i32, u32, u32, u32, u32)> {
        let i32_val = match Calendar::parse_i32(&mut str_split) {
            Some(ret) => ret,
            None => return None,
        };

        let mut u32_vec: Vec<u32> = Vec::new();
        for _i in 0..4 {
            if let Some(val) = Calendar::parse_u32(&mut str_split) {
                u32_vec.push(val);
            } else {
                return None;
            }
        }

        Some((i32_val, u32_vec[0], u32_vec[1], u32_vec[2], u32_vec[3]))
    }
}
