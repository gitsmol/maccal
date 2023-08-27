use std::fmt::Display;

use chrono::{Local, NaiveDateTime, TimeZone};

#[derive(Debug, Clone)]
pub struct CalendarItem {
    pub rowid: u32,
    pub calendar: String,
    pub start_date: NaiveDateTime,
    pub end_date: NaiveDateTime,
    pub summary: String,
    pub description: String,
    pub location: String,
    pub all_day: bool,
}

impl Display for CalendarItem {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "[rowid: {}] {} - {} -- {}\ndescription length: {} | loc: {} | all day: {}",
            self.rowid,
            self.start_date,
            self.end_date,
            self.summary,
            self.description.len(),
            self.location,
            self.all_day
        )
    }
}

impl CalendarItem {
    pub fn start_date_from_utc(&self) -> NaiveDateTime {
        Local.from_utc_datetime(&self.start_date).naive_local()
    }

    pub fn end_date_from_utc(&self) -> NaiveDateTime {
        Local.from_utc_datetime(&self.end_date).naive_local()
    }

    /// Dirname is based on startdate and time + summary
    pub fn dirname(&self) -> String {
        String::from(format!(
            "{}-{}",
            self.start_date_from_utc()
                .format("%Y-%m-%d-%H_%M")
                .to_string(),
            self.summary
        ))
    }

    /// The filename is dirname + extension (.md)
    pub fn notename(&self) -> String {
        String::from(format!("{}.md", self.dirname()))
    }

    pub fn rowid(&self) -> u32 {
        self.rowid as u32
    }
}
