use clap::Parser;
use geojson::{Feature, FeatureCollection, GeoJson, Value};
use rusqlite::{named_params, Connection, Result};
use serde::Deserialize;
use serde::Serialize;
use serde_json;
use std::convert::TryFrom;
use std::fs::File;
use std::io::prelude::*;
use std::path::Path;

// XXX This is all very quick and dirty and explody!

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[clap(value_parser)]
    name: Option<String>,

    #[clap(short, long, default_value_t = String::from("tracks.db"))]
    output: String,
}

#[derive(Clone, Serialize, Deserialize, Debug)]
struct SFLine {
    name: String,
    sf_start: (f64, f64),
    sf_end: (f64, f64),
}

fn feature_to_sfline(f: Feature) -> SFLine {
    let p = f.properties.unwrap();
    let track = p.get("name").unwrap();
    let g = f.geometry.unwrap();
    let ls = match g.value {
        Value::LineString(ls) => ls,
        _ => {
            let x: Vec<Vec<f64>> = Vec::new();
            x
        }
    };
    SFLine {
        name: track.to_string(),
        sf_start: (ls[0][0], ls[0][1]),
        sf_end: (ls[1][0], ls[1][1]),
    }
}

fn load_feature_collection(filename: &Path) -> FeatureCollection {
    let mut geojson_file = File::open(filename).unwrap();
    let mut geojson_str = String::new();
    geojson_file.read_to_string(&mut geojson_str).unwrap();

    let geojson = geojson_str.parse::<GeoJson>().unwrap();
    FeatureCollection::try_from(geojson).unwrap()
}

fn get_db(filename: &Path) -> Connection {
    match Connection::open(filename) {
        Err(e) => panic!("Failed to open database: {}", e),
        Ok(c) => {
            if let Err(e) = c.execute(
                "CREATE TABLE IF NOT EXISTS tracks (
                    id INTEGER PRIMARY KEY,
                    value TEXT NOT NULL
                )",
                [],
            ) {
                panic!("Failed to create table: {}", e)
            };
            c
        }
    }
}

fn add_track(conn: &Connection, data: SFLine) {
    let mut stmt = conn
        .prepare("INSERT INTO tracks (value) VALUES (:value)")
        .unwrap();
    stmt.execute(named_params! { ":value": serde_json::to_string(&data).unwrap()})
        .unwrap();
}

fn main() {
    let args = Args::parse();

    let filename = args.name.unwrap();
    let geojson_file = Path::new(&filename);
    let feature_collection = load_feature_collection(geojson_file);

    let filename = Path::new(&args.output);
    let conn = get_db(filename);

    for f in feature_collection.into_iter() {
        let sf = feature_to_sfline(f);
        println!("ADDING: {}", sf.name);
        add_track(&conn, sf);
    }
}
