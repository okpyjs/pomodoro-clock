#![allow(dead_code)]

use geo::geometry::Line;
use geo::Intersects;
use std::time;

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

#[derive(Debug, PartialEq)]
enum LapType {
    Out,      // outlap, has not yet crossed start/finish line
    In,       // inlap, did not cross start/finish line
    Lap(u16), // lap number
}

struct Lap {
    lap_type: LapType,
    telemetry: Vec<RbMessage>, // Telemetry datapoints for the lap
    start_time: time::Instant,
    end_time: time::Instant,
}

impl Lap {
    fn new(lap_type: LapType) -> Lap {
        // XXX should this really be the timestamp from the GPS?
        let instant = time::Instant::now();

        Lap {
            lap_type,
            telemetry: Vec::new(),
            start_time: instant,
            end_time: instant, // XXX probably an Option type
        }
    }

    // Add a telemetry point to the lap
    fn add_point(&mut self, message: RbMessage) {
        self.telemetry.push(message);
    }

    // Tests if the last data point intersects the line
    // XXX this feels odd here
    fn intersects(&self, line: Line) -> bool {
        // TODO RbMessage is specific to the racebox mini. We ultimately want
        // a generic telemetry message and can embed the geo types in that
        let coord_start_x = self.telemetry[self.telemetry.len() - 2]
            .gps_coordinates()
            .latitude();
        let coord_start_y = self.telemetry[self.telemetry.len() - 2]
            .gps_coordinates()
            .longitude();
        let coord_end_x = self.telemetry[self.telemetry.len() - 1]
            .gps_coordinates()
            .latitude();
        let coord_end_y = self.telemetry[self.telemetry.len() - 1]
            .gps_coordinates()
            .longitude();
        let prev_line = geo::Line::new(
            geo::coord! {x:coord_start_x, y:coord_start_y},
            geo::coord! {x:coord_end_x, y:coord_end_y},
        );
        prev_line.intersects(&line)
    }

    // Creates the next lap starting with the last two points
    // of the current lap (these intersect the start/finish line)
    fn next(&mut self) -> Lap {
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

        // TODO The start/end time here is not quite right since the last two locations
        // intersect the start/finish line, we will need to look at the ratio and
        // adjust the times accordingly

        // XXX should this really be the timestamp from the GPS?
        let instant = time::Instant::now();

        self.end_time = instant;

        Lap {
            lap_type,
            telemetry,
            start_time: instant,
            end_time: instant,
        }
    }

    // Current time in the lap
    fn time(&self) -> time::Duration {
        let now = time::Instant::now();
        now.duration_since(self.start_time)
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
    fn start() -> Lap {
        Lap::new(LapType::Out)
    }

    // Adds details for a completed lap and returns the next lap
    fn add_lap(&mut self, lap: Lap) -> Lap {
        self.laps.push(lap);
        let last_lap = self.laps.len() - 1;
        self.laps[last_lap].next()
    }

    // Marks the final lap as the inlap and stops the timer
    fn finish(&mut self) {
        let last_lap = self.laps.len() - 1;
        self.laps[last_lap].lap_type = LapType::In;
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
    use super::*;

    #[test]
    fn test_lap() {
        let lap = Lap::new(LapType::Out);
        assert_eq!(lap.lap_type, LapType::Out);
        assert_eq!(lap.telemetry.len(), 0);
    }

    #[test]
    fn test_track() {
        let sf_line = geo::Line::new(geo::coord! {x:1.0, y:1.0}, geo::coord! {x:2.0, y:2.0});
        let track = Track::new("Sonoma".to_string(), sf_line);
        assert_eq!(track.name, "Sonoma".to_string());
        assert_eq!(track.start_finish.start, geo::coord! {x:1.0, y:1.0});
        assert_eq!(track.start_finish.end, geo::coord! {x:2.0, y:2.0});
    }
}
