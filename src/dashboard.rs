use gtk::prelude::*;
use relm4::prelude::*;
use std::io::{self, Write};
use std::path::Path;
use tokio::sync::mpsc;

use logger::Logger;
use rbmini::connection::RbManager;
use rbmini::message::{decode_rb_message, rb_checksum, RbMessage};

const LOG_FILE: &str = "/tmp/openlaps_dashboard_testing.db";

struct DashboardApp {
    telemetry: RbMessage,
    status: String,
}

#[derive(Debug)]
enum Msg {
    Update(RbMessage),
    Status(String),
}

#[relm4::component]
impl SimpleComponent for DashboardApp {
    type Init = RbMessage;
    type Input = Msg;
    type Output = ();

    view! {
        gtk::Window {
            set_title: Some("Openlaps Dashboard"),
            set_default_size: (800, 600),
            set_fullscreened: true,

            gtk::Box {
                set_orientation: gtk::Orientation::Vertical,
                set_spacing: 5,
                set_margin_all: 5,

                gtk::Label {
                    #[watch]
                    set_label: &format!("GPS Coordinates: {}", model.telemetry.gps_coordinates()),
                    set_margin_all: 5,
                },
                gtk::Label {
                    #[watch]
                    set_label: &format!("Speed: {:.1}", model.telemetry.speed()),
                    set_margin_all: 5,
                },
                gtk::Label {
                    #[watch]
                    set_label: &format!("GPS Fix: {}", model.telemetry.is_valid_fix()),
                    set_margin_all: 5,
                },
                gtk::Label {
                    #[watch]
                    set_label: &model.status,
                    set_margin_all: 5,
                }
            }
        }
    }

    // Initialize the component.
    fn init(
        telemetry: Self::Init,
        root: &Self::Root,
        sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        let model = DashboardApp {
            telemetry,
            status: String::new(),
        };

        // Insert the code generation of the view! macro here
        let widgets = view_output!();

        // We use a seperate thread to talk to the racebox mini
        tokio::spawn(async move {
            // Connect to a racebox mini
            // XXX This is all very explode-y right now
            sender.input(Msg::Status("Setting up RaceBox mini manager".to_string()));
            let mut rb = match RbManager::new().await {
                Err(e) => {
                    panic!("{}", e);
                }
                Ok(rb) => rb,
            };

            sender.input(Msg::Status("Connecting to RaceBox mini".to_string()));
            let rc = match rb.connect().await {
                Err(e) => {
                    sender.input(Msg::Status("Failed to connect to RaceBox mini".to_string()));
                    panic!("{}", e);
                }
                Ok(conn) => conn,
            };
            sender.input(Msg::Status("".to_string()));

            // Create a logger to record telemetry to
            let logger = Logger::new(Path::new(LOG_FILE));

            let (tx, mut rx) = mpsc::channel(32);

            // Start another thread to stream from the racebox mini
            tokio::spawn(async move {
                if let Err(err) = rc.stream(tx).await {
                    panic!("{}", err)
                }
            });

            // Our receive loop, get a message from the racebox, send it to our app
            let mut checksum_failures = 0;
            while let Some(msg) = rx.recv().await {
                if !rb_checksum(&msg.value) {
                    checksum_failures += 1;
                }
                let rb_msg = decode_rb_message(&msg.value);
                // Just here to aid development
                print!("{esc}[2J{esc}[1;1H {d}", esc = 27 as char, d = rb_msg);
                print!("Checksum failures {}", checksum_failures);
                io::stdout().flush().expect("Couldn't flush stdout");

                if logger.write(&rb_msg.to_json()).is_err() {
                    continue; // do nothing for now
                }

                // Send an update message to our app
                sender.input(Msg::Update(rb_msg));
            }

            // XXX we don't have a decent way to shut down!
        });

        ComponentParts { model, widgets }
    }

    fn update(&mut self, msg: Self::Input, _sender: ComponentSender<Self>) {
        match msg {
            Msg::Update(t) => {
                self.telemetry = t;
            }
            Msg::Status(s) => {
                self.status = s;
            }
        }
    }
}

pub fn start() {
    let app = RelmApp::new("org.openlaps.dashboard");
    let telemetry = rbmini::message::RbMessage::new();
    app.run::<DashboardApp>(telemetry);
}
