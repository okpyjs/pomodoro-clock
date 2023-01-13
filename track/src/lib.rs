use rusqlite::{named_params, Connection, Result};
use serde::Deserialize;
use serde::Serialize;
use serde_json;
use std::error::Error;
use std::path::Path;

struct Tracks {
    filename: &'static Path,
    conn: Connection,
    tracks: Vec<Track>,
}

impl Tracks {
    fn new(filename: &'static Path) -> Self {
        // Open or create the tracks DB
        let conn = match Connection::open(filename) {
            Err(e) => panic!("Failed to open database: {}", e),
            Ok(c) => c,
        };
        if let Err(e) = conn.execute(
            "CREATE TABLE IF NOT EXISTS tracks (
                id INTEGER PRIMARY KEY,
                value TEXT NOT NULL
            )",
            [],
        ) {
            panic!("Failed to create table: {}", e)
        };

        let mut stmt = conn.prepare("SELECT value FROM tracks").unwrap();
        let values = stmt.query_map([], |row| Ok(row.get(0)?)).unwrap();
        let mut tracks: Vec<Track> = Vec::new();
        for value in values {
            tracks.push(serde_json::from_value(value.unwrap()).unwrap());
        }

        // XXX conn gets dropped above, this code will be refactored at some point
        let conn = match Connection::open(filename) {
            Err(e) => panic!("Failed to open database: {}", e),
            Ok(c) => c,
        };

        Tracks {
            filename,
            conn,
            tracks,
        }
    }

    fn add(
        &self,
        name: String,
        sf_start_lat: f64,
        sf_start_long: f64,
        sf_end_lat: f64,
        sf_end_long: f64,
    ) -> Result<Track, Box<dyn Error>> {
        let track = Track::new(name, sf_start_lat, sf_start_long, sf_end_lat, sf_end_long);

        // TODO handle errors!
        let mut stmt = self
            .conn
            .prepare("INSERT INTO tracks (value) VALUES (:value)")
            .unwrap();
        stmt.execute(named_params! { ":value": track.to_json()})
            .unwrap();

        Ok(track)
    }

    // Finds the track with the start/finish line closest to the coordinate
    fn find_nearest(&self, lat: f64, long: f64) -> Track {
        Track {
            name: "".to_string(),
            sf_start_lat: 0.0,
            sf_start_long: 0.0,
            sf_end_lat: 0.0,
            sf_end_long: 0.0,
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
struct Track {
    name: String,
    sf_start_lat: f64, // Start of the start/finish line
    sf_start_long: f64,
    sf_end_lat: f64, // End of the start/finish line
    sf_end_long: f64,
}

impl Track {
    fn new(
        name: String,
        sf_start_lat: f64,
        sf_start_long: f64,
        sf_end_lat: f64,
        sf_end_long: f64,
    ) -> Self {
        Track {
            name,
            sf_start_lat,
            sf_start_long,
            sf_end_lat,
            sf_end_long,
        }
    }

    pub fn to_json(&self) -> String {
        serde_json::to_string(self).unwrap()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tracks() {
        let filename = Path::new("tracks.db");
        let tracks = Tracks::new(filename);
        tracks.add("A test track".to_string(), 1.0, 1.0, 2.0, 2.0);
        tracks.find_nearest(0.0, 0.0);
    }

    #[test]
    fn test_track() {
        let track = Track::new("A test track".to_string(), 1.0, 1.0, 2.0, 2.0);
    }
}
