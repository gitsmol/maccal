use std::path::PathBuf;

use clap::Parser;
use maccal::calendar_data::{self, CalendarData};

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Name of the command
    #[arg(short, long)]
    command: String,

    /// id we are trying to fetch
    #[arg(short, long, default_value_t = 1)]
    id: u32,
}

fn init() -> CalendarData {
    CalendarData::new(PathBuf::from(
        "/Users/nme/Library/Calendars/Calendar.sqlitedb",
    ))
    .unwrap()
}

fn list() {
    let caldata = init();
    for item in caldata.cal_items {
        println!("{}", item);
    }
}

fn attendee(id: u32) {
    let caldata = init();
    if let Ok(filtered) = calendar_data::filter_attendees(caldata.db_path(), &id) {
        for item in filtered {
            println!("{}", item);
        }
    }
}

fn main() {
    let args = Args::parse();

    match args.command.as_str() {
        "list" => list(),
        "attendee" => attendee(args.id),
        _ => (),
    }
}
