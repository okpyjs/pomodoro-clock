use std::error::Error;
use std::io::{self, Write};
use tokio::sync::mpsc;

use rbmini::connection::RbManager;
use rbmini::message::{decode_rb_message, rb_checksum};

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    println!("Creating a new RbConnecting handler");
    let mut rb = match RbManager::new().await {
        Err(e) => {
            panic!("{}", e);
        }
        Ok(rb) => rb,
    };

    println!("connecting to racebox mini");
    let rc = match rb.connect().await {
        Err(e) => {
            panic!("{}", e);
        }
        Ok(conn) => conn,
    };

    let (tx, mut rx) = mpsc::channel(32);

    tokio::spawn(async move {
        if let Err(err) = rc.stream(tx).await {
            panic!("Stream failed: {}", err)
        }
    });

    let mut checksum_failures = 0;
    loop {
        while let Some(msg) = rx.recv().await {
            if !rb_checksum(&msg.value) {
                checksum_failures += 1;
            }
            let rb_msg = decode_rb_message(&msg.value);
            print!("{esc}[2J{esc}[1;1H {d}", esc = 27 as char, d = rb_msg);
            print!("Checksum failures {}", checksum_failures);
            io::stdout().flush().expect("Couldn't flush stdout");
        }
    }
}
