#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release

use eframe::{egui, CreationContext};
use std::path::Path;
use std::sync::Arc;
use std::sync::Mutex;
use std::time::Duration;
use std::{thread, time};
use tokio::runtime;
use tokio::sync::mpsc;

use logger::Logger;
use rbmini::connection::RbManager;
use rbmini::message::{decode_rb_message, RbMessage};
use timer::{Lap, LapType, Session};

use super::http;

const LOG_FILE: &str = "/tmp/openlaps_dashboard_testing.db";
const FRAME_RATE: u64 = 20; // Desired minimum FPS

macro_rules! send {
    ($ctx:ident, $model:ident, $item:ident, $value:expr) => {
        *$model.$item.lock().unwrap() = $value;
    };
}

struct DashboardModel {
    telemetry: Arc<Mutex<RbMessage>>,
    status: Arc<Mutex<String>>,
    session: Arc<Mutex<Session>>,
    lap: Arc<Mutex<Lap>>, // The current lap
}

impl DashboardModel {
    fn new() -> Self {
        let track = timer::Track::new("Default Track".to_string(), (1.0, 1.0), (2.0, 2.0));
        DashboardModel {
            telemetry: Arc::new(Mutex::new(RbMessage::new())),
            status: Arc::new(Mutex::new(String::new())),
            session: Arc::new(Mutex::new(timer::Session::new(track))),
            lap: Arc::new(Mutex::new(timer::Lap::new(LapType::Out))),
        }
    }

    fn clone(&self) -> DashboardModel {
        DashboardModel {
            telemetry: self.telemetry.clone(),
            status: self.status.clone(),
            session: self.session.clone(),
            lap: self.lap.clone(),
        }
    }
}

struct DashboardApp {
    _rt: runtime::Runtime,
    model: DashboardModel,
}

impl DashboardApp {
    fn new(ctx: &CreationContext) -> Self {
        let _rt = runtime::Builder::new_multi_thread()
            .enable_all()
            .build()
            .unwrap();

        let model = DashboardModel::new();

        let mut visuals = ctx.egui_ctx.style().visuals.clone();
        visuals.override_text_color = Some(egui::Color32::WHITE);
        ctx.egui_ctx.set_visuals(visuals);

        // Set a minimum frame rate
        let ctx_clone = ctx.egui_ctx.clone();
        _rt.spawn(async move {
            loop {
                ctx_clone.request_repaint();
                thread::sleep(time::Duration::from_millis(FRAME_RATE / 60));
            }
        });

        // Start the telemetry updater
        let ctx_clone = ctx.egui_ctx.clone();
        let model_clone = model.clone();
        _rt.spawn(async move {
            updater(ctx_clone, model_clone).await;
        });

        // Start the HTTP server
        _rt.spawn(async move {
            http::start().await;
        });
        Self { _rt, model }
    }
}

impl eframe::App for DashboardApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        let t = self.model.telemetry.lock().unwrap();
        let status = self.model.status.lock().unwrap();
        let session = self.model.session.lock().unwrap();
        let lap = self.model.lap.lock().unwrap();

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Openlaps Dashboard");

            ui.horizontal(|ui| {
                ui.label(egui::RichText::new(format!("{:.1} kph", t.speed())).size(96.0));
                ui.label(
                    egui::RichText::new(format!("Lap {}", session.current_lap_number())).size(96.0),
                );
            });

            ui.label(egui::RichText::new(pretty_duration(lap.time())).size(200.0));

            ui.label(format!("GPS Coordinates: {}", t.gps_coordinates()));
            ui.label(format!("GPS Fix: {}", t.is_valid_fix()));
            ui.label(format!("{}", status));
        });
    }
}

// For lack of a better name, this is the core logic
async fn updater(ctx: eframe::egui::Context, model: DashboardModel) {
    send!(ctx, model, status, String::from("Creating RB Manager"));
    let mut rb = match RbManager::new().await {
        Err(e) => {
            panic!("{}", e);
        }
        Ok(rb) => rb,
    };

    send!(ctx, model, status, String::from("Connecting to RB"));
    let rc = match rb.connect().await {
        Err(e) => {
            send!(ctx, model, status, e);
            panic!("Failed to connect to RB");
        }
        Ok(conn) => conn,
    };

    // Create a logger to record telemetry to
    let logger = Logger::new(Path::new(LOG_FILE));

    // Start another thread to stream from the racebox mini
    let (tx, mut rx) = mpsc::channel(32);
    let model_clone = model.clone();
    let ctx_clone = ctx.clone();
    tokio::spawn(async move {
        if let Err(err) = rc.stream(tx).await {
            send!(ctx_clone, model_clone, status, format!("{}", err));
            panic!("{}", err)
        }
    });

    send!(ctx, model, status, String::from("Running"));
    // Our receive loop, get a message from the racebox, send it to our app
    while let Some(msg) = rx.recv().await {
        let rb_msg = decode_rb_message(&msg.value);

        if logger.write(&rb_msg.to_json()).is_err() {
            continue; // do nothing for now
        }

        send!(ctx, model, telemetry, rb_msg);
    }
    // XXX we don't have a decent way to shut down!
}

fn pretty_duration(duration: Duration) -> String {
    let tenths = duration.subsec_millis() / 100;
    let sec = duration.as_secs() % 60;
    let min = (duration.as_secs() / 60) % 60;
    format!("{:0>2}:{:0>2}.{:0>1}", min, sec, tenths)
}

pub fn start() {
    // Log to stdout (if you run with `RUST_LOG=debug`).
    tracing_subscriber::fmt::init();

    // Use downlevel_defaults() to run on the Raspberry Pi 4
    let mut wgpu_options = eframe::egui_wgpu::WgpuConfiguration::default();
    wgpu_options.device_descriptor.limits = wgpu::Limits::downlevel_defaults();

    let options = eframe::NativeOptions {
        initial_window_size: Some(egui::vec2(800.0, 480.0)),
        resizable: false,
        decorated: false,
        initial_window_pos: Some(egui::pos2(0.0, 0.0)),
        always_on_top: true,
        wgpu_options,
        ..Default::default()
    };

    eframe::run_native(
        "Openlaps Dashboard",
        options,
        Box::new(|ctx| Box::new(DashboardApp::new(ctx))),
    )
}
