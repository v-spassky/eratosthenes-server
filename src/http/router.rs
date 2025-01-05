use crate::app_context::AppContext;
use crate::cli::Args;
use crate::storage::rooms::HashMapRoomsStorage;
use crate::{auth, health, http::cors, rooms};
use axum::{
    routing::{any, get, post},
    Router,
};

pub fn new(args: &Args, app_context: AppContext<HashMapRoomsStorage>) -> Router {
    let cors_policy = cors::layer(args);
    tracing::info!("Initialized HTTP configuration.");

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
        .route("/:room-id/ws", any(rooms::handlers::ws::ws));

    Router::new()
        .nest("/health", health_routes)
        .nest("/auth", auth_routes)
        .nest("/rooms", rooms_routes)
        .with_state(app_context)
        .layer(cors_policy)
        .layer(axum::middleware::from_fn(crate::middleware::tracing))
}
