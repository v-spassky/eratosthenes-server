use crate::cli::Args;
use crate::http::middleware;
use axum::{routing::any, routing::get, routing::post, Router};
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

    let cors_policy = http::init(&args);
    tracing::info!("Initialized HTTP configuration.");

    let app_context = app_context::init();
    tracing::info!("Initialized app context.");

    let health_routes = Router::new().route("/check", get(health::handlers::healthcheck));
    let auth_routes = Router::new().route("/decode-passcode", get(auth::handlers::decode_passcode));
    let users_routes = Router::new()
        .route("/", get(rooms::handlers::room::users))
        .route(
            "/:user-id/mute",
            get(rooms::handlers::host_actions::mute_user),
        )
        .route(
            "/:user-id/unmute",
            get(rooms::handlers::host_actions::unmute_user),
        )
        .route(
            "/:user-id/ban",
            post(rooms::handlers::host_actions::ban_user),
        )
        .route(
            "/:user-id/change-score",
            post(rooms::handlers::host_actions::change_user_score),
        );
    let messages_routes = Router::new().route("/", get(rooms::handlers::room::messages));
    let rooms_routes = Router::new()
        .route("/", post(rooms::handlers::room::create))
        .route(
            "/:room-id/can-connect",
            get(rooms::handlers::permissions::can_connect_to_room),
        )
        .route(
            "/:room-id/am-i-host",
            get(rooms::handlers::permissions::is_host),
        )
        .route(
            "/:room-id/save-guess",
            post(rooms::handlers::player_actions::save_guess),
        )
        .route(
            "/:room-id/submit-guess",
            post(rooms::handlers::player_actions::submit_guess),
        )
        .route(
            "/:room-id/revoke-guess",
            post(rooms::handlers::player_actions::revoke_guess),
        )
        .nest("/:room-id/users", users_routes)
        .nest("/:room-id/messages", messages_routes)
        .route("/:room-id/ws", any(rooms::handlers::ws::ws)); // TODO: Breaking change in path here.

    let app = Router::new()
        .nest("/health", health_routes)
        .nest("/auth", auth_routes)
        .nest("/rooms", rooms_routes)
        .with_state(app_context)
        .layer(cors_policy)
        .layer(axum::middleware::from_fn(middleware::tracing));

    let listener = tokio::net::TcpListener::bind(args.listen_address)
        .await
        .expect("Failed to bind to the specified address.");

    tracing::info!("Initialization completed, starting serving...");

    axum::serve(listener, app)
        .await
        .expect("Failed to run the app.");
}
