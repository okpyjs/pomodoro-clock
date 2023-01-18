use rusqlite::{named_params, Connection, Result};
use serde::Deserialize;
use serde::Serialize;
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
        &mut self,
        name: String,
        sf_start: (f64, f64),
        sf_end: (f64, f64),
    ) -> Result<Track, Box<dyn Error>> {
        let track = Track::new(name, sf_start, sf_end);

        // TODO handle errors!
        let mut stmt = self
            .conn
            .prepare("INSERT INTO tracks (value) VALUES (:value)")
            .unwrap();
        stmt.execute(named_params! { ":value": track.to_json()})
            .unwrap();

        self.tracks.push(track.clone());

        Ok(track)
    }

    // Finds the track with the start/finish line closest to the coordinate
    // We're using option because the DB may be empty
    fn find_nearest(&self, lat: f64, long: f64) -> Option<Track> {
        let mut nearest: Option<Track> = None;
        let mut distance = f64::MAX;
        for track in self.tracks.iter() {
            let d =
                ((lat - track.sf_start.0).powf(2.0) + (long - track.sf_start.1).powf(2.0)).sqrt();
            if nearest.is_none() || d < distance {
                nearest = Some(track.clone());
                distance = d;
            }
        }
        nearest
    }
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct Track {
    pub name: String,
    pub sf_start: (f64, f64), // Start of the start/finish line
    pub sf_end: (f64, f64),   // End of the start/finish line
}

impl Track {
    fn new(name: String, sf_start: (f64, f64), sf_end: (f64, f64)) -> Self {
        Track {
            name,
            sf_start,
            sf_end,
        }
    }

    pub fn to_json(&self) -> String {
        serde_json::to_string(self).unwrap()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::remove_file;

    #[test]
    fn test_tracks() {
        // Clean up any previous tests
        remove_file("tracks_rust_test.db");

        let filename = Path::new("tracks_rust_test.db");
        let mut tracks = Tracks::new(filename);
        let t = tracks
            .add("A test track".to_string(), (1.0, 1.0), (2.0, 2.0))
            .unwrap();
        assert_eq!(t.name, "A test track");
        assert_eq!(t.sf_start, (1.0, 1.0));
        assert_eq!(t.sf_end, (2.0, 2.0));

        let _ = tracks
            .add("One Magnitude".to_string(), (10.0, 10.0), (10.1, 10.1))
            .unwrap();
        let _ = tracks
            .add("Distant Track".to_string(), (100.0, 100.0), (100.1, 100.1))
            .unwrap();
        let t = tracks.find_nearest(8.0, 8.0).unwrap();
        assert_eq!(t.name, "One Magnitude");
        let _ = tracks
            .add("Just a bit closer".to_string(), (9.0, 9.0), (9.1, 9.1))
            .unwrap();
        let t = tracks.find_nearest(8.0, 8.0).unwrap();
        assert_eq!(t.name, "Just a bit closer");

        // Clean up this test
        remove_file("tracks_rust_test.db");
    }

    #[test]
    fn test_track() {
        let t = Track::new("A test track".to_string(), (1.0, 1.0), (2.0, 2.0));
        assert_eq!(t.name, "A test track");
        assert_eq!(t.sf_start, (1.0, 1.0));
        assert_eq!(t.sf_end, (2.0, 2.0));
    }
}
