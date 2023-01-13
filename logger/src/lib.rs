use rusqlite::{named_params, Connection, Result};
use serde_json;
use std::path::Path;

use rbmini::message::RbMessage;

pub struct Logger {
    path: &'static Path,
    conn: Connection,
}

impl Default for Logger {
    fn default() -> Self {
        let conn = match Connection::open_in_memory() {
            Err(e) => panic!("Failed to open in memory database: {}", e),
            Ok(c) => c,
        };
        if let Err(e) = conn.execute(
            "CREATE TABLE IF NOT EXISTS telemetry (
                id INTEGER PRIMARY KEY,
                session_id INTEGER NOT NULL,
                value TEXT NOT NULL
            )",
            [],
        ) {
            panic!("Failed to create table: {}", e)
        };

        Logger {
            path: Path::new(Path::new("")),
            conn,
        }
    }
}

impl Logger {
    pub fn new(path: &'static Path) -> Logger {
        let conn = match Connection::open(path) {
            Err(e) => panic!("Failed to open database: {}", e),
            Ok(c) => c,
        };
        if let Err(e) = conn.execute(
            "CREATE TABLE IF NOT EXISTS telemetry (
                id INTEGER PRIMARY KEY,
                session_id INTEGER NOT NULL,
                value TEXT NOT NULL
            )",
            [],
        ) {
            panic!("Failed to create table: {}", e)
        };
        Logger { path, conn }
    }

    pub fn write(&self, session_id: u64, line: &str) -> Result<(), String> {
        // TODO handle errors!
        let mut stmt = self
            .conn
            .prepare("INSERT INTO telemetry (session_id, value) VALUES (:session_id, :value)")
            .unwrap();
        stmt.execute(named_params! { ":session_id": session_id, ":value": line})
            .unwrap();
        Ok(())
    }

    pub fn close(&self) -> Result<(), String> {
        // self.conn.close(); WTF??
        Ok(())
    }

    pub fn path(&self) -> &Path {
        self.path
    }

    // Return the last row set in the telemetry table
    pub fn get_last(&self) -> Result<String> {
        let mut stmt = self
            .conn
            .prepare("SELECT value FROM telemetry ORDER BY id DESC LIMIT 1")?;
        let mut values = stmt.query_map([], |row| Ok(row.get(0)?))?;
        if let Some(value) = values.next() {
            return value;
        }
        Ok("".to_string())
    }

    // Return the list of sessions in the database
    pub fn get_sessions(&self) -> Result<Vec<u64>> {
        let mut stmt = self
            .conn
            .prepare("SELECT session_id FROM telemetry GROUP BY session_id")
            .unwrap();
        let values = stmt.query_map([], |row| Ok(row.get(0)?))?;
        let mut sessions: Vec<u64> = Vec::new();
        for value in values {
            // TODO handle errors
            sessions.push(value.unwrap());
        }
        Ok(sessions)
    }

    // Get all the data for a specific session
    // TODO create a common lap datapoint struct!
    pub fn get_session(&self, session_id: u64) -> Result<Vec<RbMessage>> {
        let mut stmt = self
            .conn
            .prepare("SELECT value FROM telemetry WHERE session_id=?")?;
        let values = stmt.query_map([session_id], |row| Ok(row.get(0)?))?;
        let mut v: Vec<RbMessage> = Vec::new();
        for value in values {
            v.push(serde_json::from_value(value.unwrap()).unwrap());
        }
        Ok(v)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new() {
        let l = Logger::default();
        assert_eq!(l.path, Path::new(""));

        // TODO something smarter than this path/filename
        let l = Logger::new(Path::new("/tmp/openlaps_test.db"));
        assert_eq!(l.path, Path::new("/tmp/openlaps_test.db"));
        assert_eq!(l.path(), l.path);
    }

    #[test]
    fn test_write() {
        let l = Logger::new(Path::new("/tmp/openlaps_test.db"));
        //assert_eq!(l.write("a line of logging"), Ok(()));
    }

    #[test]
    fn test_close() {
        let l = Logger::new(Path::new("/tmp/openlaps_test.db"));
        assert_eq!(l.close(), Ok(()));
    }

    #[test]
    fn test_get_last() {
        let l = Logger::new(Path::new("/tmp/openlaps_test.db"));
        assert_eq!(l.get_last().unwrap(), "a line of logging".to_string());
    }
}
