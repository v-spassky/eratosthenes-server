use crate::cli::Args;
use crate::http::config::CORS_POLICY;
use clap::Parser;
use warp::Filter;

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
    tracing::info!("Initialized JWT signing key.");
    let app_context = app_context::init();
    tracing::info!("Initialized app context.");
    let routes = health::endpoints::healthcheck()
        .or(auth::endpoints::decode_passcode())
        .or(rooms::endpoints::permissions::can_connect_to_room(
            &app_context,
        ))
        .or(rooms::endpoints::permissions::is_host(&app_context))
        .or(rooms::endpoints::player_actions::save_guess(&app_context))
        .or(rooms::endpoints::player_actions::submit_guess(&app_context))
        .or(rooms::endpoints::player_actions::revoke_guess(&app_context))
        .or(rooms::endpoints::host_actions::mute_user(&app_context))
        .or(rooms::endpoints::host_actions::unmute_user(&app_context))
        .or(rooms::endpoints::host_actions::ban_user(&app_context))
        .or(rooms::endpoints::host_actions::change_user_score(
            &app_context,
        ))
        .or(rooms::endpoints::room::users(&app_context))
        .or(rooms::endpoints::room::messages(&app_context))
        .or(rooms::endpoints::room::create(&app_context))
        .or(rooms::endpoints::ws::room_chat(&app_context))
        .with(CORS_POLICY.clone());
    tracing::info!("Initialization completed, starting serving...");
    warp::serve(routes).run(args.listen_address).await;
}
