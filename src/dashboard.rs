#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release

use eframe::{egui, CreationContext};
use std::path::Path;
use std::sync::Arc;
use std::sync::Mutex;
use tokio::runtime;
use tokio::sync::mpsc;

use logger::Logger;
use rbmini::connection::RbManager;
use rbmini::message::{decode_rb_message, RbMessage};

use super::http;

const LOG_FILE: &str = "/tmp/openlaps_dashboard_testing.db";

struct DashboardApp {
    _rt: runtime::Runtime,
    telemetry: Arc<Mutex<RbMessage>>,
    status: Arc<Mutex<String>>,
}

impl DashboardApp {
    fn new(ctx: &CreationContext) -> Self {
        let _rt = runtime::Builder::new_multi_thread()
            .enable_all()
            .build()
            .unwrap();

        // Start the telemetry updater
        let telemetry = Arc::new(Mutex::new(RbMessage::new()));
        let telemetry_clone = telemetry.clone();
        let status = Arc::new(Mutex::new(String::from("Initializing")));
        let status_clone = status.clone();
        let ctx_copy = ctx.egui_ctx.clone();
        _rt.spawn(async move {
            updater(ctx_copy, telemetry_clone, status_clone).await;
        });

        // Start the HTTP server
        _rt.spawn(async move {
            http::start().await;
        });
        Self {
            _rt,
            telemetry,
            status,
        }
    }
}

impl eframe::App for DashboardApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        let t = self.telemetry.lock().unwrap();
        let status = self.status.lock().unwrap();
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Openlaps Dashboard");
            ui.label(format!("Status: {}", status));
            ui.label(format!("GPS Coordinates: {}", t.gps_coordinates()));
            ui.label(format!("GPS Fix: {}", t.is_valid_fix()));
        });
    }
}

// For lack of a better name, this is the core logic
async fn updater(
    ctx: eframe::egui::Context,
    telemetry: Arc<Mutex<RbMessage>>,
    status: Arc<Mutex<String>>,
) {
    *status.lock().unwrap() = String::from("Creating RB Manager");
    ctx.request_repaint();
    let mut rb = match RbManager::new().await {
        Err(e) => {
            panic!("{}", e);
        }
        Ok(rb) => rb,
    };

    *status.lock().unwrap() = String::from("Connecting to RB");
    ctx.request_repaint();
    let status_clone = status.clone();
    let ctx_clone = ctx.clone();
    let rc = match rb.connect().await {
        Err(e) => {
            *status_clone.lock().unwrap() = e;
            ctx_clone.request_repaint();
            panic!("Failed to connect to RB");
        }
        Ok(conn) => conn,
    };

    // Create a logger to record telemetry to
    let logger = Logger::new(Path::new(LOG_FILE));

    let (tx, mut rx) = mpsc::channel(32);

    *status.lock().unwrap() = String::from("Starting RB stream");
    ctx.request_repaint();
    // Start another thread to stream from the racebox mini
    let status_clone = status.clone();
    let ctx_clone = ctx.clone();
    tokio::spawn(async move {
        if let Err(err) = rc.stream(tx).await {
            *status_clone.lock().unwrap() = format!("{}", err);
            ctx_clone.request_repaint();
            panic!("{}", err)
        }
    });

    *status.lock().unwrap() = String::from("Running");
    ctx.request_repaint();
    // Our receive loop, get a message from the racebox, send it to our app
    while let Some(msg) = rx.recv().await {
        let rb_msg = decode_rb_message(&msg.value);

        if logger.write(&rb_msg.to_json()).is_err() {
            continue; // do nothing for now
        }

        *telemetry.lock().unwrap() = rb_msg;
        ctx.request_repaint();
    }
    // XXX we don't have a decent way to shut down!
}

pub fn start() {
    // Log to stdout (if you run with `RUST_LOG=debug`).
    tracing_subscriber::fmt::init();

    let options = eframe::NativeOptions {
        initial_window_size: Some(egui::vec2(800.0, 600.0)),
        ..Default::default()
    };
    eframe::run_native(
        "Openlaps Dashboard",
        options,
        Box::new(|ctx| Box::new(DashboardApp::new(ctx))),
    )
}
