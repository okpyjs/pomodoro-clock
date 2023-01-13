use bincode::deserialize;
use chrono::LocalResult;
use chrono::TimeZone;
use chrono::Utc;
use serde::Deserialize;
use serde::Serialize;
use std::fmt;

#[allow(dead_code)]
enum FixStatus {
    NoFix = 0,
    Fix2D = 2,
    Fix3D = 3,
}

#[derive(Serialize, Deserialize, Debug)]
struct RbHeader {
    start: u16,
    class: u16,
    length: u16,
}

impl fmt::Display for RbHeader {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} {} {}", self.start, self.class, self.length)
    }
}

#[derive(Serialize, Clone, Copy, Deserialize, Debug, PartialEq, Eq)]
pub struct Coordinates {
    longitude: i32,
    latitude: i32,
}

impl fmt::Display for Coordinates {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}, {}", self.latitude(), self.longitude())
    }
}

impl Coordinates {
    pub fn longitude(&self) -> f64 {
        self.longitude as f64 / 10000000.0
    }

    pub fn latitude(&self) -> f64 {
        self.latitude as f64 / 10000000.0
    }
}

#[derive(Serialize, Clone, Copy, Deserialize, Debug, PartialEq, Eq)]
pub struct Datetime {
    pub year: u16,
    pub month: u8,
    pub day: u8,
    pub hour: u8,
    pub minute: u8,
    pub second: u8,
}

impl fmt::Display for Datetime {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match Utc.with_ymd_and_hms(
            self.year.try_into().unwrap(),
            self.month.try_into().unwrap(),
            self.day.try_into().unwrap(),
            self.hour.try_into().unwrap(),
            self.minute.try_into().unwrap(),
            self.second.try_into().unwrap(),
        ) {
            LocalResult::Single(dt) => {
                write!(f, "{}", dt)
            }
            LocalResult::Ambiguous(_dt, _dt2) => {
                write!(f, "Ambiguous datetime")
            }
            LocalResult::None => {
                write!(f, "No valid datetime")
            }
        }
    }
}

// RaceBox Mini data message sent at 25hz
// Message class 0xFF, message ID 0x01
#[derive(Serialize, Deserialize, Debug)]
pub struct RbMessage {
    // Todo: factor out the first three fields
    header: RbHeader,

    itow: u32, // number of milliseconds from the GPS week start

    /*
    Year, month, day, hour, minute, second, and nanosecond form UTC timestamp of the
    message. Note that the Nanoseconds are signed and can be negative. Month
    indexing starts from 1 for January
    */
    datetime: Datetime,

    /*
    Validity Flags
    Bit 0 - valid date
    Bit 1 - valid time
    Bit 2 - fully resolved
    Bit 3 - valid magnetic direction
    */
    validity: u8, // bitmask

    time_accuracy: u32, // nanoseconds
    nanoseconds: i32,

    /*
    Fix Status
    0 - no fix
    2 - 2d fix
    3 - 3d fix
    */
    fix_status: u8, // enum

    /*
    Fix Status Flags
    Bit 0 - valid fix
    Bit 1 - differential corrections applied
    Bit 4..2 - power state
    Bit 5 - valid heading
    Bit 7..6 - carrier phase range solution
    */
    fix_status_flags: u8, // bitmask

    /*
    Date Time Flags
    Bit 5 - available confirmation of data/time validity
    Bit 6 - confirmed UTC date validity
    Bit 7 - confirmed UTC time validity
    */
    date_time_flags: u8, // bitmask

    number_of_svs: u8, // number of space vehicles used to compute the solution

    // coordinates of the received with a factor of 10^7
    coordinates: Coordinates,

    /*
    WGS and MSL altitude: Altitude in millimetres. The WGS altitude is in the coordinate
    system of the Ellipsoid, while MSL is an approximation of altitude above Mean Sea
    Level. Note that the GPS receiver has a very limited map of the Earth’s gravitational
    field and for best results, it is recommended to implement the client-side conversion
    of WGS to MSL altitude.
    */
    wgs_altitude: i32,
    msl_altitude: i32,

    /*
    Horizontal and Vertical Accuracy: indication of the receiver’s location error in
    millimetres
    */
    horizontal_accuracy: u32,
    vertical_accuracy: u32,

    // Speed is the ground speed of the vehicle in millimetres per second.
    speed: i32,

    /*
    Heading is the direction of motion in degrees with a factor of 10^5, where zero is North
    */
    heading: i32,

    // Speed accuracy: estimation of the error of the Speed field in millimetres per second
    speed_accuracy: u32,
    heading_accuracy: u32,

    /*
    Position Dilution of Precision - indicates the error propagation of the satellite
    configuration. Usually directly related to the number of satellites. Value is with a factor
    of 100.
    */
    pdop: u16,

    /*
    Bit 0 - 1 = Invalid lat, long, wgs altitude, and msl altitude
    Bit 4..1 - Differential Correction Age
    */
    lat_lon_flags: u8, //bitmask

    /*
    contains charging status in the most significant bit (1 if charging) and
    estimation of the battery level in percentage in the remaining 7 bits.
    */
    battery_status: u8, // bitmask

    /*
    GForce X, Y, and Z - acceleration on the 3 axes in milli-g. Divide by a factor of 1000
    to convert to g values. The orientation of the axes is X - front/back, Y - right/left, Z -
    up/down
    */
    g_force_x: i16,
    g_force_y: i16,
    g_force_z: i16,

    /*
    Rotation Rate X, Y, and Z - speed of rotation on the 3 axes in centi-degrees per
    second. Divide by a factor of 100 to convert to degrees per second. The orientation
    of the axes is X - roll, Y - pitch, and Z - yaw.

    Left hand orientation
    */
    rot_rate_x: i16,
    rot_rate_y: i16,
    rot_rate_z: i16,

    checksum: RbChecksum,
}

// RaceBox Mini
#[derive(Serialize, Deserialize, Debug)]
struct RbChecksum {
    value: u16,
}

impl fmt::Display for RbChecksum {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.value)
    }
}

impl Default for RbMessage {
    fn default() -> Self {
        RbMessage {
            header: RbHeader {
                start: 0,
                class: 0,
                length: 0,
            },
            itow: 0,
            datetime: Datetime {
                year: 0,
                month: 0,
                day: 0,
                hour: 0,
                minute: 0,
                second: 0,
            },
            validity: 0,
            time_accuracy: 0,
            nanoseconds: 0,
            fix_status: 0,
            fix_status_flags: 0,
            date_time_flags: 0,
            number_of_svs: 0,
            coordinates: Coordinates {
                latitude: 0,
                longitude: 0,
            },
            wgs_altitude: 0,
            msl_altitude: 0,
            horizontal_accuracy: 0,
            vertical_accuracy: 0,
            speed: 0,
            heading: 0,
            speed_accuracy: 0,
            heading_accuracy: 0,
            pdop: 0,
            lat_lon_flags: 0,
            battery_status: 0,
            g_force_x: 0,
            g_force_y: 0,
            g_force_z: 0,
            rot_rate_x: 0,
            rot_rate_y: 0,
            rot_rate_z: 0,
            checksum: RbChecksum { value: 0 },
        }
    }
}

impl RbMessage {
    pub fn new() -> Self {
        Self::default()
    }

    // Validity Flags
    pub fn is_valid_date(&self) -> bool {
        if self.validity & 1 == 1 {
            return true;
        }
        false
    }

    pub fn is_valid_time(&self) -> bool {
        if self.validity >> 1 & 1 == 1 {
            return true;
        }
        false
    }

    pub fn is_fully_resolved(&self) -> bool {
        if self.validity >> 2 & 1 == 1 {
            return true;
        }
        false
    }

    pub fn is_valid_magnetic_declination(&self) -> bool {
        if self.validity >> 3 & 1 == 1 {
            return true;
        }
        false
    }

    // Fix Status Flags
    pub fn is_valid_fix(&self) -> bool {
        if self.fix_status_flags & 1 == 1 {
            return true;
        }
        false
    }

    pub fn is_differential_corrections_applied(&self) -> bool {
        if self.fix_status_flags >> 1 & 1 == 1 {
            return true;
        }
        false
    }

    pub fn power_state(&self) -> bool {
        // TODO
        false
    }

    pub fn is_valid_heading(&self) -> bool {
        if self.fix_status_flags >> 5 & 1 == 1 {
            return true;
        }
        false
    }

    pub fn carrier_phase_range_solution(&self) -> bool {
        // TODO
        false
    }

    // Date/Time Flags
    pub fn is_confirmation_datetime_validity(&self) -> bool {
        if self.date_time_flags >> 4 & 1 == 1 {
            return true;
        }
        false
    }

    pub fn is_confirmed_utc_date_validty(&self) -> bool {
        if self.date_time_flags >> 5 & 1 == 1 {
            return true;
        }
        false
    }

    pub fn is_confirmed_utc_time_validty(&self) -> bool {
        if self.date_time_flags >> 6 & 1 == 1 {
            return true;
        }
        false
    }

    // Lat/Lon Flags
    pub fn is_valid_position(&self) -> bool {
        if self.date_time_flags & 1 == 1 {
            return false;
        }
        true
    }

    pub fn differential_correction_age(&self) -> bool {
        // TODO
        false
    }

    pub fn gps_coordinates(&self) -> Coordinates {
        self.coordinates
    }

    // Returns speed in kph
    pub fn speed(&self) -> f32 {
        self.speed as f32 * 0.0036
    }

    pub fn to_json(&self) -> String {
        serde_json::to_string(self).unwrap()
    }

    // Not normally used, aids in testing
    pub fn update_coordinates(&mut self, longitude: i32, latitude: i32) {
        self.coordinates.longitude = longitude;
        self.coordinates.latitude = latitude;
    }

    // Getters

    pub fn datetime(&self) -> Datetime {
        self.datetime
    }

    pub fn satelites(&self) -> u8 {
        self.number_of_svs
    }

    pub fn altitude(&self) -> i32 {
        self.msl_altitude
    }

    pub fn horiz_accuracy(&self) -> u32 {
        self.horizontal_accuracy
    }

    pub fn vert_accuracy(&self) -> u32 {
        self.vertical_accuracy
    }

    pub fn heading(&self) -> i32 {
        self.heading
    }

    pub fn g_forces(&self) -> (i16, i16, i16) {
        (self.g_force_x, self.g_force_y, self.g_force_z)
    }

    pub fn rot_rates(&self) -> (i16, i16, i16) {
        (self.rot_rate_x, self.rot_rate_y, self.rot_rate_z)
    }
}

impl fmt::Display for RbMessage {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "RaceBox Mini Stream

ITOW             {itow}
Date/Time        {datetime}
Time Accuracy    {time_accuracy} {nanoseconds}
Number of svs    {num_svs}
Fix Status       {fix_status}
Fix Flags        {fix_flags}
WGS Altitude     {wgs_alt}
MSL Altitude     {msl_alt}
Accuracy        ({horiz_acc},\t{vert_acc})
Speed            {speed:.1} kph
Heading          {heading}
Speed Accuracy   {speed_acc}
Heading Accuracy {heading_acc}
PDOP             {pdop}
Lat/Long        ({coordinates})
LatG/LongG/AltG ({lat_g},\t{long_g},\t{alt_g})
RotX/RotY/RotZ  ({rot_x},\t{rot_y},\t{rot_z})
LatLong Flags   {latlong_flags}
Battery Status  {battery_status}
Header/Checksum {header} {checksum}
",
            itow = self.itow,
            datetime = self.datetime,
            time_accuracy = self.time_accuracy,
            nanoseconds = self.nanoseconds,
            num_svs = self.number_of_svs,
            fix_status = self.fix_status,
            fix_flags = self.fix_status_flags,
            wgs_alt = self.wgs_altitude,
            msl_alt = self.msl_altitude,
            horiz_acc = self.horizontal_accuracy,
            vert_acc = self.vertical_accuracy,
            speed = self.speed(),
            heading = self.heading,
            speed_acc = self.speed_accuracy,
            heading_acc = self.heading_accuracy,
            pdop = self.pdop,
            coordinates = self.coordinates,
            lat_g = self.g_force_x,
            long_g = self.g_force_y,
            alt_g = self.g_force_z,
            rot_x = self.rot_rate_x,
            rot_y = self.rot_rate_y,
            rot_z = self.rot_rate_z,
            latlong_flags = self.lat_lon_flags,
            battery_status = self.battery_status,
            header = self.header,
            checksum = self.checksum,
        )
    }
}

pub fn decode_rb_message(raw: &[u8]) -> RbMessage {
    let message: RbMessage = deserialize(raw).unwrap();
    message
}

/*
The 2-byte checksum is calculated over the packet’s contents - the message class
and ID bytes, the payload length bytes, and the payload itself. The formula is:
    // assuming Packet is a byte array containing the entire
    // packet with header and 2 spare bytes at the end for
    // the checksum:
    byte CK_A = 0, CK_B = 0
    for (int i = 2; i < len(Packet)-2; i++) {
        CK_A = CK_A + Packet[i]
        CK_B = CK_B + CK_A
    }
    Packet[len(Packet)-2] = CK_A
    Packet[len(Packet)-1] = CK_B
*/
pub fn rb_checksum(raw: &[u8]) -> bool {
    let mut ck_a: u8 = 0;
    let mut ck_b: u8 = 0;

    for byte in raw.iter().take(raw.len() - 2).skip(2) {
        (ck_a, _) = ck_a.overflowing_add(*byte);
        (ck_b, _) = ck_b.overflowing_add(ck_a)
    }

    ck_a.eq(&raw[raw.len() - 2]) && ck_b.eq(&raw[raw.len() - 1])
}

/*
Example packet

B5 62 FF 01 50 00 A0 E7 0C 07 E6 07 01 0A 08 33
08 37 19 00 00 00 2A AD 4D 0E 03 01 EA 0B C6 93
E1 0D 3B 37 6F 19 61 8C 09 00 0F 01 09 00 9C 03
00 00 2C 07 00 00 23 00 00 00 00 00 00 00 D0 00
00 00 88 A9 DD 00 2C 01 00 59 FD FF 71 00 CE 03
2F FF 56 00 FC FF 06 DB
*/
#[cfg(test)]
mod tests {
    use crate::message;

    use super::RbMessage;

    #[test]
    fn test_rb_new() {
        let msg = RbMessage::new();
        assert_eq!(msg.header.class, 0);
    }

    #[test]
    fn test_rb_checksum() {
        let raw = [
            0xB5, 0x62, 0xFF, 0x01, 0x50, 0x00, 0xA0, 0xE7, 0x0C, 0x07, 0xE6, 0x07, 0x01, 0x0A,
            0x08, 0x33, 0x08, 0x37, 0x19, 0x00, 0x00, 0x00, 0x2A, 0xAD, 0x4D, 0x0E, 0x03, 0x01,
            0xEA, 0x0B, 0xC6, 0x93, 0xE1, 0x0D, 0x3B, 0x37, 0x6F, 0x19, 0x61, 0x8C, 0x09, 0x00,
            0x0F, 0x01, 0x09, 0x00, 0x9C, 0x03, 0x00, 0x00, 0x2C, 0x07, 0x00, 0x00, 0x23, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0xD0, 0x00, 0x00, 0x00, 0x88, 0xA9, 0xDD, 0x00,
            0x2C, 0x01, 0x00, 0x59, 0xFD, 0xFF, 0x71, 0x00, 0xCE, 0x03, 0x2F, 0xFF, 0x56, 0x00,
            0xFC, 0xFF, 0x06, 0xDB,
        ];
        assert!(message::rb_checksum(&raw));
        let raw_bad_checksum = [
            0xB5, 0x62, 0xFF, 0x01, 0x50, 0x00, 0xA0, 0xE7, 0x0C, 0x07, 0xE6, 0x07, 0x01, 0x0A,
            0x08, 0x33, 0x08, 0x37, 0x19, 0x00, 0x00, 0x00, 0x2A, 0xAD, 0x4D, 0x0E, 0x03, 0x01,
            0xEA, 0x0B, 0xC6, 0x93, 0xE1, 0x0D, 0x3B, 0x37, 0x6F, 0x19, 0x61, 0x8C, 0x09, 0x00,
            0x0F, 0x01, 0x09, 0x00, 0x9C, 0x03, 0x00, 0x00, 0x2C, 0x07, 0x00, 0x00, 0x23, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0xD0, 0x00, 0x00, 0x00, 0x88, 0xA9, 0xDD, 0x00,
            0x2C, 0x01, 0x00, 0x59, 0xFD, 0xFF, 0x71, 0x00, 0xCE, 0x03, 0x2F, 0xFF, 0x56, 0x00,
            0xFC, 0xFF, 0xFF, 0xFF,
        ];
        assert!(!message::rb_checksum(&raw_bad_checksum));
    }

    #[test]
    fn test_decode_rb_message() {
        let raw = [
            0xB5, 0x62, 0xFF, 0x01, 0x50, 0x00, 0xA0, 0xE7, 0x0C, 0x07, 0xE6, 0x07, 0x01, 0x0A,
            0x08, 0x33, 0x08, 0x37, 0x19, 0x00, 0x00, 0x00, 0x2A, 0xAD, 0x4D, 0x0E, 0x03, 0x01,
            0xEA, 0x0B, 0xC6, 0x93, 0xE1, 0x0D, 0x3B, 0x37, 0x6F, 0x19, 0x61, 0x8C, 0x09, 0x00,
            0x0F, 0x01, 0x09, 0x00, 0x9C, 0x03, 0x00, 0x00, 0x2C, 0x07, 0x00, 0x00, 0x23, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0xD0, 0x00, 0x00, 0x00, 0x88, 0xA9, 0xDD, 0x00,
            0x2C, 0x01, 0x00, 0x59, 0xFD, 0xFF, 0x71, 0x00, 0xCE, 0x03, 0x2F, 0xFF, 0x56, 0x00,
            0xFC, 0xFF, 0x06, 0xDB,
        ];
        let message = message::decode_rb_message(&raw);
        assert_eq!(message.header.start, 0x62B5);
        assert_eq!(message.header.class, 0x01FF);
        assert_eq!(message.header.length, 80);
        assert_eq!(message.itow, 118286240);
        assert_eq!(
            message.datetime,
            super::Datetime {
                year: 2022,
                month: 1,
                day: 10,
                hour: 8,
                minute: 51,
                second: 8,
            }
        );
        assert_eq!(message.time_accuracy, 25);
        assert_eq!(message.nanoseconds, 239971626);
        assert_eq!(message.fix_status, 3);
        assert_eq!(message.validity, 0x37);
        assert_eq!(message.date_time_flags, 0xEA);
        assert_eq!(message.number_of_svs, 11);
        assert_eq!(message.coordinates.longitude, 232887238); // XXX 23.2887238
        assert_eq!(message.coordinates.latitude, 426719035); // XXX 42.6719035
        assert_eq!(message.wgs_altitude, 625761); // XXX 625.761
        assert_eq!(message.msl_altitude, 590095); // XXX 590.095
        assert_eq!(message.speed, 35);
        assert_eq!(message.heading, 0);
        assert_eq!(message.speed_accuracy, 208);
        assert_eq!(message.heading_accuracy, 14526856); // XXX 145.26856
        assert_eq!(message.pdop, 300); // XXX 3
        assert_eq!(message.lat_lon_flags, 0x00);
        assert_eq!(message.battery_status, 89);
        assert_eq!(message.g_force_x, -3); // XXX scaled by 1000
        assert_eq!(message.g_force_y, 113);
        assert_eq!(message.g_force_z, 974);
        assert_eq!(message.rot_rate_x, -209); // ??? 2.09 deg/sec
        assert_eq!(message.rot_rate_y, 86);
        assert_eq!(message.rot_rate_z, -4); // ??? 0.04 deg/sec
        assert_eq!(message.checksum.value, 0xDB06);
    }

    #[test]
    fn test_valid_date() {
        let raw = [
            0xB5, 0x62, 0xFF, 0x01, 0x50, 0x00, 0xA0, 0xE7, 0x0C, 0x07, 0xE6, 0x07, 0x01, 0x0A,
            0x08, 0x33, 0x08, 0x37, 0x19, 0x00, 0x00, 0x00, 0x2A, 0xAD, 0x4D, 0x0E, 0x03, 0x01,
            0xEA, 0x0B, 0xC6, 0x93, 0xE1, 0x0D, 0x3B, 0x37, 0x6F, 0x19, 0x61, 0x8C, 0x09, 0x00,
            0x0F, 0x01, 0x09, 0x00, 0x9C, 0x03, 0x00, 0x00, 0x2C, 0x07, 0x00, 0x00, 0x23, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0xD0, 0x00, 0x00, 0x00, 0x88, 0xA9, 0xDD, 0x00,
            0x2C, 0x01, 0x00, 0x59, 0xFD, 0xFF, 0x71, 0x00, 0xCE, 0x03, 0x2F, 0xFF, 0x56, 0x00,
            0xFC, 0xFF, 0x06, 0xDB,
        ];
        let message = message::decode_rb_message(&raw);
        // TODO confirm the bits in the example packet
        assert!(message.is_valid_date());
        assert!(message.is_valid_time());
        assert!(message.is_fully_resolved());
        assert!(!message.is_valid_magnetic_declination());
    }

    #[test]
    fn test_fix_status_flags() {
        let raw = [
            0xB5, 0x62, 0xFF, 0x01, 0x50, 0x00, 0xA0, 0xE7, 0x0C, 0x07, 0xE6, 0x07, 0x01, 0x0A,
            0x08, 0x33, 0x08, 0x37, 0x19, 0x00, 0x00, 0x00, 0x2A, 0xAD, 0x4D, 0x0E, 0x03, 0x01,
            0xEA, 0x0B, 0xC6, 0x93, 0xE1, 0x0D, 0x3B, 0x37, 0x6F, 0x19, 0x61, 0x8C, 0x09, 0x00,
            0x0F, 0x01, 0x09, 0x00, 0x9C, 0x03, 0x00, 0x00, 0x2C, 0x07, 0x00, 0x00, 0x23, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0xD0, 0x00, 0x00, 0x00, 0x88, 0xA9, 0xDD, 0x00,
            0x2C, 0x01, 0x00, 0x59, 0xFD, 0xFF, 0x71, 0x00, 0xCE, 0x03, 0x2F, 0xFF, 0x56, 0x00,
            0xFC, 0xFF, 0x06, 0xDB,
        ];
        let message = message::decode_rb_message(&raw);
        // TODO confirm the bits in the example packet
        assert!(message.is_valid_fix());
        assert!(!message.is_differential_corrections_applied());
        assert!(!message.is_valid_heading());
    }

    #[test]
    fn test_datetime_flags() {
        let raw = [
            0xB5, 0x62, 0xFF, 0x01, 0x50, 0x00, 0xA0, 0xE7, 0x0C, 0x07, 0xE6, 0x07, 0x01, 0x0A,
            0x08, 0x33, 0x08, 0x37, 0x19, 0x00, 0x00, 0x00, 0x2A, 0xAD, 0x4D, 0x0E, 0x03, 0x01,
            0xEA, 0x0B, 0xC6, 0x93, 0xE1, 0x0D, 0x3B, 0x37, 0x6F, 0x19, 0x61, 0x8C, 0x09, 0x00,
            0x0F, 0x01, 0x09, 0x00, 0x9C, 0x03, 0x00, 0x00, 0x2C, 0x07, 0x00, 0x00, 0x23, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0xD0, 0x00, 0x00, 0x00, 0x88, 0xA9, 0xDD, 0x00,
            0x2C, 0x01, 0x00, 0x59, 0xFD, 0xFF, 0x71, 0x00, 0xCE, 0x03, 0x2F, 0xFF, 0x56, 0x00,
            0xFC, 0xFF, 0x06, 0xDB,
        ];
        let message = message::decode_rb_message(&raw);
        // TODO confirm the bits in the example packet
        assert!(!message.is_confirmation_datetime_validity());
        assert!(message.is_confirmed_utc_date_validty());
        assert!(message.is_confirmed_utc_time_validty());
    }

    #[test]
    fn test_latlon_flags() {
        let raw = [
            0xB5, 0x62, 0xFF, 0x01, 0x50, 0x00, 0xA0, 0xE7, 0x0C, 0x07, 0xE6, 0x07, 0x01, 0x0A,
            0x08, 0x33, 0x08, 0x37, 0x19, 0x00, 0x00, 0x00, 0x2A, 0xAD, 0x4D, 0x0E, 0x03, 0x01,
            0xEA, 0x0B, 0xC6, 0x93, 0xE1, 0x0D, 0x3B, 0x37, 0x6F, 0x19, 0x61, 0x8C, 0x09, 0x00,
            0x0F, 0x01, 0x09, 0x00, 0x9C, 0x03, 0x00, 0x00, 0x2C, 0x07, 0x00, 0x00, 0x23, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0xD0, 0x00, 0x00, 0x00, 0x88, 0xA9, 0xDD, 0x00,
            0x2C, 0x01, 0x00, 0x59, 0xFD, 0xFF, 0x71, 0x00, 0xCE, 0x03, 0x2F, 0xFF, 0x56, 0x00,
            0xFC, 0xFF, 0x06, 0xDB,
        ];
        let message = message::decode_rb_message(&raw);
        // TODO confirm the bits in the example packet
        assert!(message.is_valid_position());
    }

    #[test]
    fn test_gps_coordinates() {
        let raw = [
            0xB5, 0x62, 0xFF, 0x01, 0x50, 0x00, 0xA0, 0xE7, 0x0C, 0x07, 0xE6, 0x07, 0x01, 0x0A,
            0x08, 0x33, 0x08, 0x37, 0x19, 0x00, 0x00, 0x00, 0x2A, 0xAD, 0x4D, 0x0E, 0x03, 0x01,
            0xEA, 0x0B, 0xC6, 0x93, 0xE1, 0x0D, 0x3B, 0x37, 0x6F, 0x19, 0x61, 0x8C, 0x09, 0x00,
            0x0F, 0x01, 0x09, 0x00, 0x9C, 0x03, 0x00, 0x00, 0x2C, 0x07, 0x00, 0x00, 0x23, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0xD0, 0x00, 0x00, 0x00, 0x88, 0xA9, 0xDD, 0x00,
            0x2C, 0x01, 0x00, 0x59, 0xFD, 0xFF, 0x71, 0x00, 0xCE, 0x03, 0x2F, 0xFF, 0x56, 0x00,
            0xFC, 0xFF, 0x06, 0xDB,
        ];
        let message = message::decode_rb_message(&raw);
        // TODO confirm the bits in the example packet
        assert_eq!(
            message.gps_coordinates(),
            super::Coordinates {
                latitude: 426719035,
                longitude: 232887238
            }
        );
        assert_eq!(message.gps_coordinates().longitude(), 23.2887238);
        assert_eq!(message.gps_coordinates().latitude(), 42.6719035);
    }

    #[test]
    fn test_speed() {
        let raw = [
            0xB5, 0x62, 0xFF, 0x01, 0x50, 0x00, 0xA0, 0xE7, 0x0C, 0x07, 0xE6, 0x07, 0x01, 0x0A,
            0x08, 0x33, 0x08, 0x37, 0x19, 0x00, 0x00, 0x00, 0x2A, 0xAD, 0x4D, 0x0E, 0x03, 0x01,
            0xEA, 0x0B, 0xC6, 0x93, 0xE1, 0x0D, 0x3B, 0x37, 0x6F, 0x19, 0x61, 0x8C, 0x09, 0x00,
            0x0F, 0x01, 0x09, 0x00, 0x9C, 0x03, 0x00, 0x00, 0x2C, 0x07, 0x00, 0x00, 0x23, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0xD0, 0x00, 0x00, 0x00, 0x88, 0xA9, 0xDD, 0x00,
            0x2C, 0x01, 0x00, 0x59, 0xFD, 0xFF, 0x71, 0x00, 0xCE, 0x03, 0x2F, 0xFF, 0x56, 0x00,
            0xFC, 0xFF, 0x06, 0xDB,
        ];
        let message = message::decode_rb_message(&raw);
        assert_eq!(message.speed(), 0.126);
    }
}
