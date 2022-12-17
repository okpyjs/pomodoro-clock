use gtk::prelude::*;
use relm4::prelude::*;
use std::io::{self, Write};
use tokio::sync::mpsc;

use rbmini::connection::RbManager;
use rbmini::message::{decode_rb_message, rb_checksum, RbMessage};

struct DashboardApp {
    telemetry: RbMessage,
}

#[derive(Debug)]
enum Msg {
    Update(RbMessage),
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

            gtk::Box {
                set_orientation: gtk::Orientation::Vertical,
                set_spacing: 5,
                set_margin_all: 5,

                gtk::Label {
                    #[watch]
                    set_label: &format!("Latitude: {}", model.telemetry.gps_coordinates().0),
                    set_margin_all: 5,
                },
                gtk::Label {
                    #[watch]
                    set_label: &format!("Longitude: {}", model.telemetry.gps_coordinates().1),
                    set_margin_all: 5,
                },
            }
        }
    }

    // Initialize the component.
    fn init(
        telemetry: Self::Init,
        root: &Self::Root,
        _sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        let model = DashboardApp { telemetry };

        // Insert the code generation of the view! macro here
        let widgets = view_output!();

        tokio::spawn(async move {
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
        });

        ComponentParts { model, widgets }
    }

    fn update(&mut self, msg: Self::Input, _sender: ComponentSender<Self>) {
        match msg {
            Msg::Update(t) => {
                self.telemetry = t;
            }
        }
    }
}

#[tokio::main]
async fn main() {
    let app = RelmApp::new("org.openlaps.dashboard");

    let telemetry = rbmini::message::RbMessage::new();

    app.run::<DashboardApp>(telemetry);
}
