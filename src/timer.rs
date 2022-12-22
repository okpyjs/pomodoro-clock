#![allow(dead_code)]

use rbmini::message::RbMessage;

// The core model and implementation of a lap timer
//
// The ugly business of all of this is crossing the start/finish line..
// similarly with sectors.
//
// When two telemetry datapoints intersect with the start/finish line we
// know a lap has finished and another has begun. The new lap will need to
// include these two datapoints as well. We can use the ratio of line
// segment's two halves to get a more accurate lap time.

enum LapType {
    Out,      // outlap, has not yet crossed start/finish line
    In,       // inlap, did not cross start/finish line
    Lap(u16), // lap number
}

struct Lap {
    lap_type: LapType,
    telemetry: Vec<RbMessage>, // Telemetry datapoints for the lap
}

impl Lap {
    fn new(lap_type: LapType) -> Lap {
        Lap {
            lap_type,
            telemetry: Vec::new(),
        }
    }

    // Add a telemetry point to the lap
    fn add_point(&mut self, message: RbMessage) {
        self.telemetry.push(message);
    }

    // Creates the next lap starting with the last two points
    // of the current lap (these intersect the start/finish line)
    fn next(&self) -> Lap {
        let lap_type = match self.lap_type {
            LapType::Out => LapType::Lap(1),
            LapType::Lap(num) => LapType::Lap(num + 1),
            LapType::In => LapType::In,
        };
        let telemetry: Vec<RbMessage> = Vec::new();
        let (_a, _b) = match &self.telemetry[..] {
            [.., a, b] => (a, b),
            _ => panic!("array shorter than 2"), // XXX
        };
        // TODO Implement copy on RbMessage
        //telemetry.push(a.clone());
        //telemetry.push(b.clone());
        Lap {
            lap_type,
            telemetry,
        }
    }
}

struct Session {
    track: Track,   // The track this session took place at
    laps: Vec<Lap>, // List of laps
}

impl Session {
    // Create a new session
    fn new(track: Track) -> Session {
        Session {
            track,
            laps: Vec::new(),
        }
    }

    // Starts the timer on the outlap
    fn start() {}

    // Adds details for a completed lap
    fn add_lap(&mut self, lap: Lap) {
        self.laps.push(lap);
    }

    // Marks the final lap as the inlap and stops the timer
    fn finish() {}
}

struct Coordinates {
    latitude: f32,
    longitude: f32,
}

impl Coordinates {
    // Create a new coordinate
    fn new(latitude: f32, longitude: f32) -> Coordinates {
        Coordinates {
            latitude,
            longitude,
        }
    }
}

// A georgraphic line segment
struct Line {
    start: Coordinates,
    end: Coordinates,
}

impl Line {
    // Create a new geographic line segment
    fn new(start: Coordinates, end: Coordinates) -> Line {
        Line { start, end }
    }

    // Returns true if first and second coordinates intersect the line
    fn intersects(_first: Coordinates, _second: Coordinates) -> bool {
        false
    }
}

struct Sector {
    start: Line, // Beginning of the sector
    end: Line,   // End of the sector
}

impl Sector {
    // Create a new sector on the track
    fn new(start: Line, end: Line) -> Sector {
        Sector { start, end }
    }
}

struct Track {
    name: String,         // Name of the track and configuration
    start_finish: Line,   // Start/Finish line coordinates
    sectors: Vec<Sector>, // List of track sectors
}

impl Track {
    // Creates a new track
    fn new(name: String, start_finish: Line) -> Track {
        Track {
            name,
            start_finish,
            sectors: Vec::new(),
        }
    }

    // Add a new sector to the track
    fn add_sector(&mut self, sector: Sector) {
        self.sectors.push(sector);
    }
}

#[cfg(test)]
mod tests {
    use super::Coordinates;

    #[test]
    fn test_coordinates() {
        let c = Coordinates::new(1.234, 5.678);
        assert_eq!(c.latitude, 1.234);
        assert_eq!(c.longitude, 5.678);
    }
}
