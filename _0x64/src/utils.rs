use crate::cal::Calendar;

use clap::{arg, Parser, AppSettings};
use std::io::{Split, Write};
use std::path::Path;
use std::process::exit;
use std::{fs, io};
use std::cmp::min;
use std::fs::File;
use std::os::unix::fs::OpenOptionsExt;

struct AddEventArg {
    n: String,
    d: Option<String>,
    da: (i32, u32, u32),
    ti: (u32, u32),
}

struct RepeatEventArg {
    n: String,
    d: Option<String>,
    r: String,
    dt: (i32, u32, u32, u32, u32),
    //dt2: (i32, u32, u32, u32, u32), //todo (allow deletion of multiple events)
}

struct ModifyEventArg {
    id: u16,
    n: String,
    d: Option<String>,
}

#[derive(Parser)]
#[clap(
name = "0x64",
author = "author: c928",
about = "CLI tool to plan 100 days ahead",
long_about = None,
global_setting(AppSettings::DeriveDisplayOrder),
)]
struct Args {
    #[clap(
        short = 'I',
        long,
        takes_value = false,
        required = false,
        help = "If specified, 0x64 will enter interactive mode"
    )]
    interactive: bool,

    #[clap(
        short,
        long,
        takes_value = true,
        required = false,
        default_value = "default.cal",
        value_name = "FILE",
        help = "Use an existing calendar file"
    )]
    calendar: String,

    #[clap(
        short,
        long,
        takes_value = true,
        help = "Specify the starting date of the challenge (YYYY-MM-DD).\n\
        Must be set when creating a new calendar"
    )]
    init_date: Option<String>,

    #[clap(
        short = 'o',
        long,
        takes_value = true,
        required = false,
        value_name = "FILE",
        help = "Remove the specified calendar"
    )]
    remove_cal: Option<String>,

    #[clap(
        short = 'b',
        long,
        takes_value = true,
        value_name = "FILE",
        help = "Retrieve last backup made the last time when 0x64 was executed.\n\
        To revert the change, reuse the same command. The destination file\nmust be specified as argument"
    )]
    retrieve_backup: Option<String>,

    #[clap(
        short,
        long,
        takes_value = true,
        //value_parser = parse_add_event,
        help = "Add a new event on a specific day/time.\nArg syntax must be \
        name|description|YYYY-MM-DD-hh-mm.\nIf the event has no description, leave it empty\n\
        (name||YYYY-...)",
    )]
    add_event: Option<String>,

    #[clap(
        short = 'R',
        long,
        takes_value = true,
        help = "Add a repeating event from a certain date.\nArg syntax must be:\n\
        name|description|su-mo-tu-we-th-fr-sa|YYYY-MM-DD-hh-mm\nwhere su, mo, tu... \
        are the days when the event occurs.\nIf the event occurs every day of the week,\n\
        use name|description|*|YYYY-MM-DD-hh-mm syntax.\nIf the event has no description, \
        leave it empty\n(name||...)"
    )]
    add_repeating: Option<String>,

    #[clap(
        short,
        long,
        takes_value = true,
        help = "Modify an already existing event using its id. Arg syntax must be\n\
        new_name|new_description|event_id. If the event has no description,\n leave it empty \
        (new_name||event_id)"
    )]
    modify_event: Option<String>,

    #[clap(
        short,
        long,
        takes_value = true,
        //value_parser = todo
        help = "Remove an existing event by specifying its event_id"
    )]
    //pub remove_event: Option<Vec<u16>>, todo (allow deletion of multiple events)
    remove_event: Option<u16>,

    #[clap(
        short,
        long,
        value_name = "DISPLAY_TYPE",
        help = "List all events from a specific file\n\
                0 - display by date: ascending\n\
                1 - by date: descending (default)\n\
                2 - by name: ascending\n\
                3 - by name: descending\n\
                4 - by date of addition: ascending\n\
                5 - by date of addition: descending\n"
    )]
    display_events: Option<u8>, //todo default to 1
}

fn e_name_check<'a>(e_name: &'a str, arg_name: &'a str) -> u8 {
    let len = e_name.len();
    if len > 32 {
        eprintln!("Error: Event name must have <= 32 characters");
        return 1;
    } else if len == 0 {
        eprintln!(
            "Error: Event name must be specified in {}",
            arg_name
        );
        return 1;
    }

    0
}

fn e_desc_check(e_desc: &str) -> u8 {
    if e_desc == "" {
        return 2;
    } else if e_desc.len() > 256 {
        eprintln!("Error: Event description must have <= 256 characters");
        return 1;
    }

    0
}

fn e_date_time_check<'a>(
    e_dt: &'a str,
    arg_name: &'a str,
) -> Result<(i32, u32, u32, u32, u32), ()> {
    return match Calendar::parse_date_time(e_dt.split("-")) {
        Some(ret) => Ok(ret),
        None => {
            eprintln!("Error: Parsing date/time from {}", arg_name);
            return Err(());
        }
    };
}

//todo: improve cal file checking ?
pub fn is_cal_file(filename: &str) -> bool {
    let len = filename.len();
    if len < 5 {
        return false;
    }

    return match &filename[len-4..len] {
        ".cal" => true,
        _ => false,
    };
}

fn parse_add_event(s: &str, interactive_mode: bool) -> Result<AddEventArg, ()> {
    let cmd_str = match interactive_mode {
        true => "add command",
        false => "--add-event argument",
    };

    let mut s: Vec<&str> = s.split("|").collect();
    if s.len() != 3 {
        eprintln!("Error: Parsing {}", cmd_str);
        return Err(());
    }

    let mut e_arg = AddEventArg {
        n: "".to_owned(),
        d: None,
        da: (0, 0, 0),
        ti: (0, 0),
    };

    e_arg.n = match e_name_check(s[0], cmd_str) {
        0 => s[0].to_owned(),
        _ => return Err(()),
    };

    e_arg.d = match e_desc_check(s[1]) {
        0 => Some(s[1].to_owned()),
        2 => None,
        _ => return Err(()),
    };

    let dt = match e_date_time_check(s[2], cmd_str) {
        Ok(ret) => ret,
        Err(_) => return Err(()),
    };

    e_arg.da = (dt.0, dt.1, dt.2);
    e_arg.ti = (dt.3, dt.4);

    Ok(e_arg)
}

//todo: add optional last date (when event stops repeating)
fn parse_add_repeating_event(s: &str, interactive_mode: bool) -> Result<RepeatEventArg, ()> {
    let cmd_str = match interactive_mode {
        true => "add-rep command",
        false => "--add-repeating argument",
    };

    let mut s: Vec<&str> = s.split("|").collect();
    if s.len() != 4 {
        eprintln!("Error: Parsing {}", cmd_str);
        return Err(());
    }

    let mut e_rep_arg = RepeatEventArg {
        n: "".to_owned(),
        d: None,
        r: "".to_owned(),
        dt: (0, 0, 0, 0, 0),
    };

    e_rep_arg.n = match e_name_check(s[0], cmd_str) {
        0 => s[0].to_owned(),
        _ => return Err(()),
    };

    e_rep_arg.d = match e_desc_check(s[1]) {
        0 => Some(s[1].to_owned()),
        2 => None,
        _ => return Err(()),
    };

    let len = s[2].len();
    if len > 20 || len == 0 {
        eprintln!(
            "Error: Repeating pattern must be specified and cannot be longer than 20 characters"
        );
        return Err(());
    }
    e_rep_arg.r = s[2].to_owned();

    e_rep_arg.dt = match e_date_time_check(s[3], cmd_str) {
        Ok(ret) => ret,
        Err(_) => return Err(()),
    };

    Ok(e_rep_arg)
}

fn parse_modify_event(s: &str, interactive_mode: bool) -> Result<ModifyEventArg, ()> {
    let cmd_str = match interactive_mode {
        true => "modify command",
        false => "--modify-event argument",
    };

    let mut s: Vec<&str> = s.split("|").collect();
    if s.len() != 3 {
        eprintln!("Error: Parsing {}", cmd_str);
        return Err(());
    }

    let mut e_mod_arg = ModifyEventArg {
        id: 0,
        n: "".to_owned(),
        d: None,
    };

    e_mod_arg.n = match e_name_check(s[0], cmd_str) {
        0 => s[0].to_owned(),
        _ => return Err(()),
    };

    e_mod_arg.d = match e_desc_check(s[1]) {
        0 => Some(s[1].to_owned()),
        2 => None,
        _ => return Err(()),
    };

    e_mod_arg.id = match s[2].parse::<u16>() {
        Ok(id) => id,
        Err(_) => {
            eprintln!("Error: Parsing event id from {}", cmd_str);
            return Err(());
        }
    };

    Ok(e_mod_arg)
}

fn init_cal_interactive_mode(
    mut file_contents: &mut String,
    filename: String,
    init_date: &str,
) -> Option<Calendar> {
    if backup_cal(&filename) == 1 {
        return None;
    }

    let init_d = if init_date == "" {
        None
    } else {
        Some(init_date.to_string())
    };

    let mut cal = match Calendar::new(filename, init_d) {
        Some(c) => c,
        None => return None,
    };

    if cal.get_cal_contents(&mut file_contents) == 1 {
        return None;
    }

    if cal.parse_calendar(&file_contents) == 1 {
        return None;
    }

    Some(cal)
}

fn backup_cal(filename: &String) -> u8 {
    /*
    let backup_file = match rm_backup {
        true => ".backup-rm.cal",
        false => ".backup-mod.cal",
    };
    */

    if Path::new(filename).exists() {
        if let Err(_) = fs::copy(filename, ".backup.cal") {
            eprintln!(
                "Error: Copying '{}' failed. Check file permissions and try again",
                filename
            );
            return 1;
        }
    }

    0
}

fn remove_cal(filename: &str) -> u8 {
    let len = filename.len();
    if !Path::new(filename).exists() {
        eprintln!("Error: '{}' does not exist", filename);
        return 1;
    }

    if !is_cal_file(filename) {
        eprintln!("Error: '{}' does not seem to be a calendar file. Please remove it manually.", filename);
        return 1;
    }

    if let Err(_) = fs::remove_file(filename) {
        eprintln!("Error: '{}' could not be removed (check file permissions)", filename);
        return 1;
    }

    println!("[+] '{}' successfully removed", filename);
    0
}

fn retrieve_backup(filename: &String) -> u8 {
    if !Path::new(".backup.cal").exists() {
        eprintln!("Error: no backup file found");
        return 1;
    } else if !Path::new(filename).exists() {
        if let Err(_) = File::create(filename) {
            //eprintln!("Error: Creating '{}'", )
        }
    }

    let nums = [2, 0, 1]; //todo
    let filenames = [".backup.cal", filename, ".backup_tmp"];
    for i in 0..3 {
        if let Err(_) = fs::rename(filenames[i], filenames[nums[i]]) {
            eprintln!("Error: Renaming '{}'", filenames[i]);
            return 1;
        }
    }

    0
}

pub fn run_0x64() -> u8 {
    let args = Args::parse();

    // can't use && because of ownership :(
    if args.interactive {
        if run_interactive(args) == 1 {
            return 1;
        }
    } else if run_non_interactive(args) == 1 {
        return 1;
    }

    0
}

fn run_non_interactive(args: Args) -> u8 {
    if let Some(arg_str) = args.retrieve_backup {
        if retrieve_backup(&arg_str) == 1 {
            return 1;
        }
        println!("[+] Backup successfully retrieved to '{}'", arg_str);
    } else {
        if backup_cal(&args.calendar) == 1 {
            return 1;
        }
    }

    if let Some(arg_str) = args.remove_cal {
        if remove_cal(&arg_str) == 1 {
            return 1;
        }
    }

    println!("[+] Chosen filename: {}", args.calendar);
    let mut cal = match Calendar::new(args.calendar, args.init_date) {
        Some(c) => c,
        None => return 1,
    };

    let mut file_contents = String::new();
    if cal.get_cal_contents(&mut file_contents) == 1 {
        return 1;
    }

    if cal.parse_calendar(&file_contents) == 1 {
        return 1;
    }

    if let Some(arg_str) = args.add_event {
        match parse_add_event(&arg_str, false) {
            Ok(arg) => {
                if cal.add_event('x', arg.n, arg.d, arg.da, arg.ti, true) == 1 {
                    return 1;
                }
            }
            Err(_) => return 1,
        };
    }

    if let Some(arg_str) = args.add_repeating {
        match parse_add_repeating_event(&arg_str, false) {
            Ok(arg) => {
                if cal.add_repeating_event(arg.r, 'x', arg.n, arg.d, arg.dt) == 1 {
                    return 1;
                }
            }
            Err(_) => return 1,
        }
    }

    if let Some(arg_str) = args.modify_event {
        match parse_modify_event(&arg_str, false) {
            Ok(arg) => {
                if cal.modify_event(arg.id, arg.n, arg.d) == 1 {
                    return 1;
                }
            }
            Err(_) => return 1,
        }
    }

    if let Some(arg_u16) = args.remove_event {
        //todo: use Option for removed_evt
        if cal.remove_event(arg_u16, &mut "-1".to_owned(), true) == 1 {
            return 1;
        }
    }

    if let Some(arg_u8) = args.display_events {
        if arg_u8 > 5 {
            eprintln!("Error: Display type must be a number between 0 and 5 (--help for more)");
            return 1;
        } else if cal.display_events(arg_u8) == 1 {
            return 1;
        }
    }

    0
}

fn run_interactive(mut args: Args) -> u8 {
    print!(
        "[+] 0x64 interactive mode\n\
    Type 'help' or 'h' to view the list of available commands.\n\
    Current calendar: '{}'\n",
        args.calendar
    );

    // todo: also, try to use static array for words (of length 2)
    let stdin = io::stdin();
    let mut stdout = io::stdout();
    let mut input = String::new();
    let mut cal = Calendar::new_empty_cal();
    let mut file_contents = String::new();
    const CMD_PARSING_ERR: &str = "Error: Parsing command. Type 'help' or 'h' for more.";
    const CAL_INIT_ERR: &str = "Error: Calendar must be initiated (init command) before";
    loop {
        print!("> ");
        if let Err(_) = stdout.flush() {
            eprintln!("Error: Could not flush stdout");
            return 1;
        }

        input.clear();
        if let Err(_) = stdin.read_line(&mut input) {
            eprintln!("Error: Could not read from stdin");
            return 1;
        }

        //todo: manage case where name or desc has whitespace
        let words: Vec<&str> = input.trim().split_whitespace().collect();
        let w_cnt = words.len();
        if w_cnt == 0 {
            continue;
        } else if w_cnt > 3 {
            eprintln!("{CMD_PARSING_ERR}");
            continue;
        }

        match words[0] {
            "exit" | "quit" | "q" => break,
            "help" | "h" => {
                println!(
                    "\thelp, h: show this text\n\n\
            \texit, quit, q: exit the program\n\n\
            \tcal, file <filename>: specify the calendar file used\n\n\
            \tinit [YYYY-MM-DD]: initialize the calendar with the chosen filename\n\
            \t\t(the starting date of the challenge must be specified when\n\t\tcreating a new calendar file)\n\n\
            \tremove-cal, rm-cal <filename>: remove the specified calendar\n\n\
            \tbackup, back: retrieve last backup (made when removing or initiating\n\t\t a cal)\n\n\
            \tadd <name|desc|YYYY-MM-DD-hh-mm>: add a new event (description can\n\t\t be left empty)\n\n\
            \tadd-rep <name|desc|su-mo-tu-we-th-fr-sa|YYYY-MM-DD-hh-mm>: add a\n\t\trepeating \
            event (su-mo-tu... are the days when the event\n\t\toccurs. use '*' to specify every \
            day of the week)\n\n\
            \tmodify, mod <new_name|name_desc|event_id>: modify an already\n\
            \t\texisting event\n\n\
            \tremove, rm <event_id>: remove an event by specifying its id\n\n\
            \tdisplay, dis <display_type>: list all events from the chosen file\n\
            \t\t0 - display by date: ascending\n\
            \t\t1 - by date: descending (default)\n\
            \t\t2 - by name: ascending\n\
            \t\t3 - by name: descending\n\
            \t\t4 - by date of addition: ascending\n\
            \t\t5 - by date of addition: descending\n"
                );
            },
            "cal" | "file" => {
                if w_cnt != 2 {
                    eprintln!("{CMD_PARSING_ERR}");
                    continue;
                }

                let len = words[1].len();
                if !is_cal_file(words[1]) {
                    eprintln!("Error: '{}' is not a calendar file (.cal extension)", words[1]);
                    continue;
                }

                // println!("[+] Chosen filename: {}", words[1]);
                args.calendar = words[1].to_string();
            },
            "init" => {
                let file_exist = Path::new(&args.calendar).exists();
                if w_cnt > 2 {
                    eprintln!("{CMD_PARSING_ERR}");
                    continue;
                } else if w_cnt == 1 && !file_exist {
                    eprintln!(
                        "Error: starting date must be specified because '{}' does\nnot \
                    currently exist",
                        args.calendar
                    );
                    continue;
                }

                let mut init_date = "";
                if w_cnt == 2 {
                    if file_exist {
                        let mut yes_no = String::new();
                        loop {
                            print!(
                                "'{}' already exists. Would you like to overwrite it? [Y/n]: ",
                                args.calendar
                            );
                            if let Err(_) = stdout.flush() {
                                eprintln!("Error: Could not flush stdout");
                                return 1;
                            }

                            yes_no.clear();
                            if let Err(_) = stdin.read_line(&mut yes_no) {
                                eprintln!("Error: Reading from stdin");
                                return 1;
                            }
                            let rm = match &yes_no.to_lowercase()[0..yes_no.len()-1] {
                                "" | "y" | "yes" => true,
                                "n" | "no" => false,
                                _ => continue,
                            };

                            if rm {
                                backup_cal(&args.calendar);
                                if let Err(_) = fs::remove_file(&args.calendar) {
                                    eprintln!("Error: '{}' could not be removed (check file permissions)", args.calendar);
                                    continue;
                                }
                            }

                            break;
                        }
                    }
                    init_date = words[1];
                }

                if let Some(cal_ret) = init_cal_interactive_mode(&mut file_contents, args.calendar.clone(), init_date) {
                    cal = cal_ret;
                } else {
                    continue;
                }
            },
            "remove-cal" | "rm-cal" => {
                if w_cnt != 2 {
                    eprintln!("{CMD_PARSING_ERR}");
                    continue;
                }

                if remove_cal(words[1]) == 1 {
                    continue;
                }
            },
            "backup" | "back" => {
                if w_cnt != 1 {
                    eprintln!("{CMD_PARSING_ERR}");
                    continue;
                }

                if retrieve_backup(&args.calendar) == 1 {
                    continue;
                }

                println!("[+] Backup retrieved to '{}'.\nTo revert the change, reuse the same command.", args.calendar);
            },
            "add" => {
                if !cal.is_empty() {
                    if w_cnt != 2 {
                        eprintln!("{CMD_PARSING_ERR}");
                        continue;
                    }

                    match parse_add_event(words[1], true) {
                        Ok(arg) => {
                            if cal.add_event('x', arg.n, arg.d, arg.da, arg.ti, true) == 1 {
                                continue;
                            }
                        }
                        Err(_) => continue,
                    };
                } else {
                    eprintln!("{} adding an event", CAL_INIT_ERR);
                    continue;
                }
            },
            "add-rep" => {
                if !cal.is_empty() {
                    if w_cnt != 2 {
                        eprintln!("{CMD_PARSING_ERR}");
                        continue;
                    }

                    match parse_add_repeating_event(words[1], true) {
                        Ok(arg) => {
                            if cal.add_repeating_event(arg.r, 'x', arg.n, arg.d, arg.dt) == 1 {
                                continue;
                            }
                        }
                        Err(_) => continue,
                    }
                } else {
                    eprintln!("{} adding an event", CAL_INIT_ERR);
                    continue;
                }
            },
            "modify" | "mod" => {
                if !cal.is_empty() {
                    if w_cnt != 2 {
                        eprintln!("{CMD_PARSING_ERR}");
                        continue;
                    }

                    match parse_modify_event(words[1], true) {
                        Ok(arg) => {
                            if cal.modify_event(arg.id, arg.n, arg.d) == 1 {
                                continue;
                            }
                        }
                        Err(_) => continue,
                    }
                } else {
                    eprintln!("{} modifying an event", CAL_INIT_ERR);
                    continue;
                }
            },
            "remove" | "rm" => {
                if !cal.is_empty() {
                    if w_cnt != 2 {
                        eprintln!("{CMD_PARSING_ERR}");
                        continue;
                    }

                    if let Ok(e_id) = words[1].parse::<u16>() {
                        if cal.remove_event(e_id, &mut "-1".to_owned(), true) == 1 {
                            continue;
                        }
                    } else {
                        eprintln!("Error: Parsing event id");
                        continue;
                    }
                } else {
                    eprintln!("{} removing an event", CAL_INIT_ERR);
                    continue;
                }
            },
            "display" | "dis" => {
                if !cal.is_empty() {
                    if w_cnt != 2 {
                        eprintln!("{CMD_PARSING_ERR}");
                        continue;
                    }

                    if let Ok(print_type) = words[1].parse::<u8>() {
                        if print_type > 5 {
                            eprintln!("Error: Print type must be a number between 0 and 5 (type help for more)");
                            continue;
                        }

                        if cal.display_events(print_type) == 1 {
                            continue;
                        }
                    } else {
                        eprintln!("Error: Parsing print type");
                        continue;
                    }
                } else {
                    eprintln!("{} displaying events", CAL_INIT_ERR);
                    continue;
                }
            },
            _ => {
                eprintln!("{CMD_PARSING_ERR}");
                continue;
            }
        }
    }

    0
}
