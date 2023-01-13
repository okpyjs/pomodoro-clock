use std::error::Error;

struct Tracks {
    filename: String,
}

impl Tracks {
    fn new(filename: String) -> Self {
        // Open or create the tracks DB
        Tracks { filename }
    }

    fn add(
        &self,
        name: String,
        sf_start: (f64, f64),
        sf_end: (f64, f64),
    ) -> Result<Track, Box<dyn Error>> {
        Ok(Track::new(name, sf_start, sf_end))
    }

    // Finds the track with the start/finish line closest to the coordinate
    fn find_nearest(&self, lat: f64, long: f64) -> Track {
        Track {
            name: "".to_string(),
            sf_start: (0.0, 0.0),
            sf_end: (0.0, 0.0),
        }
    }
}

struct Track {
    name: String,
    sf_start: (f64, f64), // Start of the start/finish line
    sf_end: (f64, f64),   // End of the start/finish line
}

impl Track {
    fn new(name: String, sf_start: (f64, f64), sf_end: (f64, f64)) -> Self {
        Track {
            name,
            sf_start,
            sf_end,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tracks() {
        let tracks = Tracks::new("tracks.db".to_string());
        tracks.add("A test track".to_string(), (1.0, 1.0), (2.0, 2.0));
        tracks.find_nearest(0.0, 0.0);
    }

    #[test]
    fn test_track() {
        let track = Track::new("A test track".to_string(), (1.0, 1.0), (2.0, 2.0));
    }
}
