mod utils;
mod cal;

fn main() {
    if utils::run_0x64() == 1 {
        eprintln!("Exiting 0x64");
        return;
    }
}

//todo  - display events in terminal | ok but display it on a specific day/week/month
//todo  - add repeating events | ok but write in one line and add one (with an id) when modifying it
//        (and subtract from the line)
//todo  - improve args parsing + run_non_interactive() | ok
//todo  - make backup of file before editing it | ok
//todo  - run_interactive() | ok
//todo  - file encryption + integrity check | blowfish (or aes?)
//todo  - gui or export event list as pdf ?
//todo - restructure project (args (or cli), utils, cal, main)
//todo  - comment + test everything

//todo: mkdir for cals + add --list-cals + .backup-{calendar_name}.cal (1 backup for each cal)
//todo: if pwd != cals { ... }
//todo: --retrieve-backup ?

//todo: clear command in interactive mode ?
//todo: allow removing multiple events at once (3214 or 234,23421,53 or *)
//todo: keyboard to reuse last command ?
//todo: use unwrap for certain functions instead of printing errors (set_len(0)) ?