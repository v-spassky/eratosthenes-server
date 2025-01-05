use crate::cli::Args;
use crate::http::middleware;
use clap::Parser;

mod app_context;
mod auth;
mod cli;
mod health;
mod http;
mod logging;
mod map_locations;
mod rooms;
mod storage;
mod users;

#[tokio::main]
async fn main() {
    let args = Args::parse();

    logging::init(&args);
    tracing::info!("Initialized logging layers.");

    auth::init(&args);
    tracing::info!("Initialized HMAC code.");

    let app_context = app_context::init();
    tracing::info!("Initialized app context.");

    let routes = http::router::new(&args, app_context);

    let listener = tokio::net::TcpListener::bind(args.listen_address)
        .await
        .expect("Failed to bind to the specified address.");

    tracing::info!("Initialization completed, starting serving...");

    axum::serve(listener, routes)
        .await
        .expect("Failed to run the app.");
}
