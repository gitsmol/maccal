use std::path::PathBuf;

use rusqlite::{Connection, Result};

#[derive(Debug, Clone)]
pub struct Attendee {
    identity_id: u32,
    pub email: String,
    pub phone_number: String,
    pub status: u32,
}

impl Attendee {
    pub fn id(&self) -> &u32 {
        &self.identity_id
    }
}

#[derive(Debug, Clone)]
pub struct AttendeeList {
    pub attendees: Vec<Attendee>,
}

impl AttendeeList {
    pub fn new(path: &PathBuf, rowid: u32) -> Result<Self> {
        let conn = Connection::open(path)?;
        let mut stmt = conn.prepare(&format!(
            "SELECT identity_id, email, phone_number, status FROM Participant WHERE owner_id = '{}'",
            rowid
        ))?;

        let items = stmt.query_map([], |row| {
            Ok(Attendee {
                identity_id: match row.get(0) {
                    Ok(res) => res,
                    Err(_) => 0,
                },
                email: match row.get(1) {
                    Ok(res) => res,
                    Err(_) => String::from("<NA>"),
                },
                phone_number: match row.get(2) {
                    Ok(res) => res,
                    Err(_) => String::from("<NA>"),
                },
                status: match row.get(3) {
                    Ok(res) => res,
                    Err(_) => 0,
                },
            })
        })?;

        let mut attendee_list = vec![];
        for item in items {
            attendee_list.push(item?);
        }

        Ok(Self {
            attendees: attendee_list,
        })
    }
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use crate::calendar_data::CalendarData;

    use super::AttendeeList;

    #[test]
    fn test_create_attendees() {
        let testdb_path = PathBuf::from("./testdata/Calendar.sqlitedb");
        let caldata = CalendarData::new(testdb_path.to_owned()).unwrap();

        for item in &caldata.cal_items {
            let _ = AttendeeList::new(&testdb_path, item.rowid);
        }
    }
}
