fn main() {
    println!("Lap Timer demo");
    let track = timer::Track::new("Test Track".to_string(), (2.01, 0.01), (2.01, 2.01));
    let mut session = timer::Session::new(track);

    let mut lap = session.start();
    println!("Session started");

    println!("On lap {:?}", lap.number());

    let points = vec![
        (1.0, 1.0),
        (1.0, 2.0),
        (1.0, 3.0),
        (1.0, 4.0),
        (2.0, 4.0),
        (3.0, 4.0),
        (4.0, 4.0),
        (4.0, 3.0),
        (4.0, 2.0),
        (4.0, 1.0),
        (3.0, 1.0),
        (2.0, 1.0),
        (1.0, 1.0),
        (1.1, 1.1),
        (1.1, 2.1),
        (1.1, 3.1),
        (1.1, 4.1),
        (2.1, 4.1),
        (3.1, 4.1),
        (4.1, 4.1),
        (4.1, 3.1),
        (4.1, 2.1),
        (4.1, 1.1),
        (3.1, 1.1),
        (2.1, 1.1),
    ];

    for point in points {
        let p = lap.add_point(point.0, point.1);
        println!("Added point {:?} at {:?}", p.coord(), p.at());
        if session.is_lap_complete(&lap) {
            println!("Lap finished!");
            lap = session.add_lap(lap);
            println!("Starting {:?}", lap.number());
        }
    }

    println!("Session finished");
}
