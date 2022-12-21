use hyper::service::{make_service_fn, service_fn};
use hyper::Server;
use hyper::{Body, Request, Response};
use std::convert::Infallible;
use std::net::SocketAddr;
use std::path::Path;

use logger::Logger;

const LOG_FILE: &str = "/tmp/openlaps_dashboard_testing.db";

pub async fn handle(_: Request<Body>) -> Result<Response<Body>, Infallible> {
    // Create a logger to record telemetry to
    // XXX this is only temporary, needs to be passed as part of context
    let logger = Logger::new(Path::new(LOG_FILE));
    let value = logger.get_last().unwrap();
    Ok(Response::new(value.into()))
}

pub async fn start() {
    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    let make_svc = make_service_fn(|_conn| async { Ok::<_, Infallible>(service_fn(handle)) });
    let server = Server::bind(&addr).serve(make_svc);

    if let Err(e) = server.await {
        eprintln!("server error: {}", e);
    }
}
