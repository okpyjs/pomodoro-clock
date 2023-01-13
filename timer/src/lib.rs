#![allow(dead_code)]

use geo::geometry::Line;
use geo::Intersects;
use geo::{coord, Coord};
use std::time;

// The core model and implementation of a lap timer
//
// The ugly business of all of this is crossing the start/finish line..
// similarly with sectors.
//
// When two telemetry datapoints intersect with the start/finish line we
// know a lap has finished and another has begun. The new lap will need to
// include these two datapoints as well. We can use the ratio of line
// segment's two halves to get a more accurate lap time.

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum LapType {
    Out,      // outlap, has not yet crossed start/finish line
    In,       // inlap, did not cross start/finish line
    Lap(u16), // lap number
}

#[derive(Clone)]
pub struct Point {
    coord: Coord,
    time: time::Instant,
}

impl Point {
    fn new(lat: f64, long: f64) -> Self {
        Point {
            coord: coord! {x:lat, y:long},
            time: time::Instant::now(),
        }
    }

    pub fn coord(&self) -> (f64, f64) {
        (self.coord.x, self.coord.y)
    }

    pub fn at(&self) -> time::Instant {
        self.time
    }
}

pub struct Lap {
    lap_type: LapType,
    points: Vec<Point>, // Sequence of coordinates for the lap
    start_time: time::Instant,
    end_time: time::Instant,
}

impl Lap {
    pub fn new(lap_type: LapType) -> Lap {
        // XXX should this really be the timestamp from the GPS?
        let instant = time::Instant::now();

        Lap {
            lap_type,
            points: Vec::new(),
            start_time: instant,
            end_time: instant, // XXX probably an Option type
        }
    }

    pub fn copy(&self) -> Lap {
        Lap {
            lap_type: self.lap_type,
            points: self.points.to_vec(),
            start_time: self.start_time,
            end_time: self.end_time,
        }
    }

    // Add a telemetry point to the lap
    pub fn add_point(&mut self, lat: f64, long: f64) -> &Point {
        // XXX we're abusing time here, this should be supplied
        let point = Point::new(lat, long);
        self.points.push(point);
        self.points.last().unwrap()
    }

    // Tests if the last data point intersects the line
    // XXX this feels odd here
    fn intersects(&self, line: Line) -> bool {
        if self.points.len() < 2 {
            return false; // We don't have at least 2 points to work with
        }
        let start = &self.points[self.points.len() - 2];
        let end = &self.points[self.points.len() - 1];
        let prev_line = geo::Line::new(start.coord, end.coord);
        prev_line.intersects(&line)
    }

    // Creates the next lap starting with the last two points
    // of the current lap (these intersect the start/finish line)
    pub fn next_lap(&mut self) -> Lap {
        let lap_type = match self.lap_type {
            LapType::Out => LapType::Lap(1),
            LapType::Lap(num) => LapType::Lap(num + 1),
            LapType::In => LapType::In,
        };
        if self.points.len() < 2 {
            panic!("array shorter than 2");
        }

        // TODO The start/end time here is not quite right since the last two locations
        // intersect the start/finish line, we will need to look at the ratio and
        // adjust the times accordingly

        // XXX this is wrong but convenient to start the next lap
        // It's possible the final point itself intersects with the line!
        // This would lead to double counting.
        let points = vec![self.points.pop().unwrap()];

        // XXX should this really be the timestamp from the GPS?
        let instant = time::Instant::now();

        self.end_time = instant;

        Lap {
            lap_type,
            points,
            start_time: instant,
            end_time: instant,
        }
    }

    // Current time in the lap
    pub fn time(&self) -> time::Duration {
        let now = time::Instant::now();
        now.duration_since(self.start_time)
    }

    pub fn number(&self) -> &LapType {
        &self.lap_type
    }
}

pub struct Session {
    track: Track,   // The track this session took place at
    laps: Vec<Lap>, // List of laps
}

impl Session {
    // Create a new session
    pub fn new(track: Track) -> Session {
        Session {
            track,
            laps: Vec::new(),
        }
    }

    // Starts the timer on the outlap
    pub fn start(&self) -> Lap {
        Lap::new(LapType::Out)
    }

    // Adds details for a completed lap and returns the next lap
    pub fn add_lap(&mut self, lap: Lap) -> Lap {
        self.laps.push(lap);
        let last_lap = self.laps.len() - 1;
        self.laps[last_lap].next_lap()
    }

    // Marks the final lap as the inlap and stops the timer
    fn finish(&mut self) {
        let last_lap = self.laps.len() - 1;
        self.laps[last_lap].lap_type = LapType::In;
    }

    pub fn is_lap_complete(&self, lap: &Lap) -> bool {
        if lap.intersects(self.track.start_finish) {
            return true;
        }
        false
    }

    pub fn current_lap_number(&self) -> usize {
        self.laps.len()
    }
}

struct Sector {
    start: Line, // Beginning of the sector
    end: Line,   // End of the sector
}

impl Sector {
    // Create a new sector on the track
    pub fn new(start: Line, end: Line) -> Sector {
        Sector { start, end }
    }
}

pub struct Track {
    name: String,         // Name of the track and configuration
    start_finish: Line,   // Start/Finish line coordinates
    sectors: Vec<Sector>, // List of track sectors
}

impl Track {
    // Creates a new track
    pub fn new(name: String, sf_start: (f64, f64), sf_end: (f64, f64)) -> Track {
        Track {
            name,
            start_finish: geo::Line::new(
                geo::coord! {x:sf_start.0, y:sf_start.1},
                geo::coord! {x:sf_end.0, y:sf_end.1},
            ),
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
        assert_eq!(lap.points.len(), 0);
    }

    #[test]
    fn test_session() {
        let sf_line = geo::Line::new(geo::coord! {x:2.1, y:1.0}, geo::coord! {x:2.6, y:4.0});
        let track = Track::new("Sonoma".to_string(), (2.1, 1.0), (2.6, 4.0));

        let session = Session::new(track);
        assert_eq!(session.laps.len(), 0);

        let mut lap = session.start();
        assert_eq!(lap.lap_type, LapType::Out);

        lap.add_point(1.0, 1.0);
        assert!(!lap.intersects(sf_line)); // No intersection

        lap.add_point(2.0, 2.0);
        assert!(!lap.intersects(sf_line)); // No intersection

        lap.add_point(3.0, 3.0);
        assert!(lap.intersects(sf_line)); // Intersection

        // Move to the next lap since we crossed start/finish
        lap = lap.next_lap();
        assert!(matches!(lap.lap_type, LapType::Lap(1)));

        lap.add_point(4.0, 4.0);
        assert!(!lap.intersects(sf_line)); // No intersection
    }

    #[test]
    fn test_track() {
        let track = Track::new("Sonoma".to_string(), (1.0, 1.0), (2.0, 2.0));
        assert_eq!(track.name, "Sonoma".to_string());
        assert_eq!(track.start_finish.start, geo::coord! {x:1.0, y:1.0});
        assert_eq!(track.start_finish.end, geo::coord! {x:2.0, y:2.0});
    }
}
