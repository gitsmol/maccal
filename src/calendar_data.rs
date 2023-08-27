use std::path::PathBuf;

use crate::calendar_data;

use super::calendaritem::CalendarItem;
use chrono::{Duration, Local, NaiveDate};
use log::{debug, info};
use rusqlite::{Connection, Statement};

pub struct CalendarData {
    db_path: PathBuf,
    pub cal_items: Vec<CalendarItem>,
    start_date: NaiveDate,
    end_date: NaiveDate,
}

/// Struct containing calendaritems from a Calendar.app sqlite file.
impl CalendarData {
    /// Creates a new CalendarData object, populated with a `Vec<CalendarItem>`
    pub fn new(db_path: PathBuf) -> rusqlite::Result<Self> {
        info!("Creating new CalendarData object");
        let now = chrono::Local::now().date_naive();
        let start_date = now - Duration::days(1);
        let end_date = now + Duration::weeks(12);
        match calendar_data::get_items(&db_path, start_date, end_date) {
            Ok(cal_items) => Ok(CalendarData {
                db_path,
                cal_items,
                start_date,
                end_date,
            }),
            Err(e) => Err(e),
        }
    }

    pub fn refresh(&mut self) -> rusqlite::Result<()> {
        match crate::calendar_data::get_items(&self.db_path, self.start_date, self.end_date) {
            Ok(items) => {
                self.cal_items = items;
                Ok(())
            }
            Err(e) => Err(e),
        }
    }

    /// Set the startdate earlier by a number of weeks and refresh the data
    pub fn set_startdate_weeks_earlier(&mut self, weeks: i64) -> rusqlite::Result<()> {
        self.start_date -= Duration::weeks(weeks);
        self.refresh()
    }

    /// Set the end date later by a number of weeks and refresh the data
    pub fn set_enddate_weeks_later(&mut self, weeks: i64) -> rusqlite::Result<()> {
        self.end_date += Duration::weeks(weeks);
        self.refresh()
    }

    // Reset to default start and end dates (12 weeks including today)
    pub fn set_default_start_end_dates(&mut self) -> rusqlite::Result<()> {
        let now = chrono::Local::now().date_naive();
        self.start_date = now - Duration::days(1);
        self.end_date = now + Duration::weeks(12);
        self.refresh()
    }

    pub fn db_path(&self) -> &PathBuf {
        &self.db_path
    }
}

pub fn get_items(
    db_path: &PathBuf,
    start_date: NaiveDate,
    end_date: NaiveDate,
) -> rusqlite::Result<Vec<CalendarItem>> {
    info!("Fetching CalendarItems from database.");
    let conn = Connection::open(db_path)?;
    let start_fmt = start_date.format("%Y-%m-%d").to_string();
    let end_fmt = end_date.format("%Y-%m-%d").to_string();

    debug!("Selecting dates between {} and {}", start_fmt, end_fmt);

    let mut stmt = conn.prepare(
        format!(
            "
        SELECT
	a.rowid,
		b.title AS calendar,
	datetime (start_date,
		'unixepoch',
		'31 years', '1 day') AS start_date,
	datetime (end_date,
		'unixepoch',
		'31 years') AS end_date,
	summary,
	description,
	c.title,
	all_day
        FROM
	CalendarItem a
	LEFT JOIN Calendar b ON calendar_id = b.ROWID
	LEFT JOIN Location c ON a.location_id = c.ROWID
        WHERE
	datetime (start_date, 'unixepoch', '31 years') BETWEEN DATE('{start_fmt}')
	AND DATE('{end_fmt}')
        ORDER BY
	start_date ASC,
	all_day",
        )
        .as_str(),
    )?;

    calendar_data::query_to_item(&mut stmt)
}

pub fn filter_attendees(
    db_path: &PathBuf,
    attendee_id: &u32,
) -> rusqlite::Result<Vec<CalendarItem>> {
    info!(
        "Running SELECT query for all items with identity_id {}",
        attendee_id
    );

    let conn = Connection::open(&db_path)?;
    let mut stmt = conn.prepare(
        format!(
            "SELECT DISTINCT
	b.rowid,
	c.title AS calendar,
	datetime (b.start_date, 'unixepoch', '31 years', '1 day') AS start_date,
	datetime (b.end_date, 'unixepoch', '31 years') AS end_date,
	b.summary,
	b.description,
	l.title AS location,
	b.all_day
                FROM
	Participant p
	LEFT JOIN CalendarItem b ON p.owner_id = b.ROWID
	LEFT JOIN Calendar c ON b.calendar_id = c.ROWID
	LEFT JOIN Location l ON b.location_id = l.ROWID
                WHERE
	p.identity_id = {}
	ORDER BY
	start_date DESC;",
            attendee_id
        )
        .as_str(),
    )?;

    calendar_data::query_to_item(&mut stmt)
}

fn query_to_item(stmt: &mut Statement<'_>) -> rusqlite::Result<Vec<CalendarItem>> {
    info!("Reading SQL SELECT query to Vec<CalendarItem>");
    let items = stmt.query_map([], |row| {
        let rowid: u32 = row.get(0).unwrap();
        let summary: String = row.get(4).unwrap_or_default();
        debug!("\nROWID = {}\nsummary = {}", rowid, summary);
        Ok(CalendarItem {
            rowid: match row.get(0) {
                Ok(res) => res,
                Err(_) => 0,
            },
            calendar: match row.get(1) {
                Ok(res) => res,
                Err(_) => String::from("<NA>"),
            },
            start_date: match row.get(2) {
                Ok(res) => res,
                Err(_) => Local::now().naive_local(),
            },
            end_date: match row.get(3) {
                Ok(res) => res,
                Err(_) => Local::now().naive_local(),
            },
            summary: match row.get(4) {
                Ok(res) => res,
                Err(_) => String::from("<NA>"),
            },
            description: match row.get(5) {
                Ok(res) => res,
                Err(_) => String::from("<NA>"),
            },
            location: match row.get(6) {
                Ok(res) => res,
                Err(_) => String::from("<NA>"),
            },
            all_day: match row.get(7) {
                Ok(res) => res,
                Err(_) => false,
            },
        })
    })?;

    let mut cal_items = vec![];
    for item in items {
        cal_items.push(item?);
    }

    Ok(cal_items)
}

#[cfg(test)]
mod tests {
    // Importing names from outer (for mod tests) scope.
    use super::*;

    #[test]
    fn test_create_caldata() {
        let caldata = CalendarData::new(PathBuf::from("./testdata/Calendar.sqlitedb")).unwrap();
        assert!(caldata.cal_items.len() > 0);
    }

    #[test]
    fn test_filter_attendees() {
        let caldata = CalendarData::new(PathBuf::from("./testdata/Calendar.sqlitedb")).unwrap();
        let filtered_items = match calendar_data::filter_attendees(&caldata.db_path, &7) {
            Ok(res) => res,
            Err(e) => panic!("{}", e),
        };
        assert!(caldata.cal_items.len() > 0);
        assert!(filtered_items.len() > 0);
    }
}
