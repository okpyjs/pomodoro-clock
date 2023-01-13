use std::path::Path;

use logger::Logger;
fn main() {
    println!("Creating logger");
    let logger = Logger::new(Path::new("/tmp/openlaps_test.db"));

    println!("Logging to {:?}", logger.path());
    /*if let Err(err) = logger.write("A line of logging") {
        panic!("{}", err);
    }*/

    println!("Closing the log");
    if let Err(err) = logger.close() {
        panic!("{}", err);
    }

    println!("Done!");
}
