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

struct DashboardModel {
    telemetry: Arc<Mutex<RbMessage>>,
    status: Arc<Mutex<String>>,
}

impl DashboardModel {
    fn new() -> Self {
        DashboardModel {
            telemetry: Arc::new(Mutex::new(RbMessage::new())),
            status: Arc::new(Mutex::new(String::new())),
        }
    }

    fn clone(&self) -> DashboardModel {
        DashboardModel {
            telemetry: self.telemetry.clone(),
            status: self.status.clone(),
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
        // Start the telemetry updater

        let ctx_copy = ctx.egui_ctx.clone();
        let model_copy = model.clone();
        _rt.spawn(async move {
            updater(ctx_copy, model_copy).await;
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
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Openlaps Dashboard");
            ui.label(format!("Status: {}", status));
            ui.label(format!("GPS Coordinates: {}", t.gps_coordinates()));
            ui.label(format!("GPS Fix: {}", t.is_valid_fix()));
        });
    }
}

// For lack of a better name, this is the core logic
async fn updater(ctx: eframe::egui::Context, model: DashboardModel) {
    *model.status.lock().unwrap() = String::from("Creating RB Manager");
    ctx.request_repaint();
    let mut rb = match RbManager::new().await {
        Err(e) => {
            panic!("{}", e);
        }
        Ok(rb) => rb,
    };

    *model.status.lock().unwrap() = String::from("Connecting to RB");
    ctx.request_repaint();
    //let model_clone = model.clone();
    let ctx_clone = ctx.clone();
    let rc = match rb.connect().await {
        Err(e) => {
            *model.status.lock().unwrap() = e;
            ctx_clone.request_repaint();
            panic!("Failed to connect to RB");
        }
        Ok(conn) => conn,
    };

    // Create a logger to record telemetry to
    let logger = Logger::new(Path::new(LOG_FILE));

    let (tx, mut rx) = mpsc::channel(32);

    *model.status.lock().unwrap() = String::from("Starting RB stream");
    ctx.request_repaint();
    // Start another thread to stream from the racebox mini
    let model_clone = model.clone();
    let ctx_clone = ctx.clone();
    tokio::spawn(async move {
        if let Err(err) = rc.stream(tx).await {
            *model_clone.status.lock().unwrap() = format!("{}", err);
            ctx_clone.request_repaint();
            panic!("{}", err)
        }
    });

    *model.status.lock().unwrap() = String::from("Running");
    ctx.request_repaint();
    // Our receive loop, get a message from the racebox, send it to our app
    while let Some(msg) = rx.recv().await {
        let rb_msg = decode_rb_message(&msg.value);

        if logger.write(&rb_msg.to_json()).is_err() {
            continue; // do nothing for now
        }

        *model.telemetry.lock().unwrap() = rb_msg;
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
