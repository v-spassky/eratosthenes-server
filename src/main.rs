use crate::app_context::{AppContext, RequestContext};
use crate::auth::handlers::AuthHttpHandler;
use crate::cli::Args;
use crate::health::handlers::HealthHttpHandler;
use crate::logging::{consts::DEFAULT_CLIENT_IP, QuickwitLoggingLayerBuilder};
use crate::query_params::{UserIdsQueryParams, UsernameQueryParam};
use crate::rooms::handlers::http::{CreateRoomHttpHandler, RoomHttpHandler};
use crate::rooms::handlers::ws::RoomWsHandler;
use crate::storage::rooms::HashMapRoomsStorage;
use crate::users::handlers::UsersHttpHandler;
use clap::Parser;
use map_locations::models::LatLng;
use std::collections::HashMap;
use std::convert::Infallible;
use std::net::SocketAddr;
use std::time::Duration;
use std::time::{SystemTime, UNIX_EPOCH};
use tokio::task;
use tokio::time::Instant;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;
use url::Url;
use warp::http::header::{HeaderMap, HeaderValue};
use warp::{hyper::Method, ws::Ws, Filter};

mod app_context;
mod auth;
mod cli;
mod health;
mod logging;
mod map_locations;
mod query_params;
mod rooms;
mod storage;
mod users;

#[tokio::main]
async fn main() {
    let args = Args::parse();

    let quickwit_url = Url::parse(&args.quickwit_url).unwrap();
    let quickwit_logging_layer = QuickwitLoggingLayerBuilder::new(quickwit_url)
        .marker_field("task")
        .map_marker_to_index("http_request", "http_requests")
        .map_marker_to_index("client_sent_ws_message", "client_sent_ws_messages")
        .map_marker_to_index("sockets_count", "sockets_counts")
        .with_batch_size(100)
        .build();
    tracing_subscriber::registry()
        .with(quickwit_logging_layer)
        .init();

    let app_context = AppContext::<HashMapRoomsStorage>::default();
    let app_context_in_sockets_logger = app_context.clone();

    task::spawn(async move {
        loop {
            tokio::time::sleep(Duration::from_secs(10)).await;
            let count = app_context_in_sockets_logger.sockets.count().await;
            let timestamp = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs();
            tracing::info!(task = "sockets_count", count = count, timestamp = timestamp);
        }
    });

    let mut headers = HeaderMap::new();
    headers.insert("Content-Type", HeaderValue::from_static("application/json"));

    let cors = warp::cors()
        .allow_origin("http://127.0.0.1:3000")
        .allow_origin("http://localhost:3000")
        .allow_origin("https://eratosthenes.vercel.app/")
        .allow_headers(vec![
            "User-Agent",
            "Sec-Fetch-Mode",
            "Referer",
            "Origin",
            "Access-Control-Request-Method",
            "Access-Control-Request-Headers",
            "content-type",
            "Public-ID",
            "Private-ID",
        ])
        .allow_methods(&[Method::POST, Method::GET, Method::OPTIONS])
        .build();

    let rooms = warp::path("rooms")
        .and(warp::ws())
        .and(warp::path::param::<String>())
        .and(warp::query::<UserIdsQueryParams>())
        .and(warp::addr::remote())
        .map({
            let app_context = app_context.clone();
            move |ws: Ws,
                  room_id,
                  query_params: UserIdsQueryParams,
                  client_ip: Option<SocketAddr>| {
                let app_context = app_context.clone();
                let request_context = RequestContext {
                    public_id: query_params.public_id,
                    private_id: query_params.private_id,
                    room_id,
                    client_ip,
                };
                ws.on_upgrade(|socket| async {
                    RoomWsHandler::new(app_context, request_context, socket)
                        .await
                        .on_user_connected()
                        .await
                })
            }
        })
        .with(cors.clone());

    let can_connect_to_room = warp::path("rooms")
        .and(warp::path::param::<String>())
        .and(warp::path("can-connect"))
        .and(warp::query::<UsernameQueryParam>())
        .and(warp::header::<String>("Public-ID"))
        .and(warp::header::<String>("Private-ID"))
        .and(warp::addr::remote())
        .and_then({
            let app_context = app_context.clone();
            move |room_id,
                  query_params: UsernameQueryParam,
                  public_id,
                  private_id,
                  client_ip: Option<SocketAddr>| {
                let app_context = app_context.clone();
                let request_context = RequestContext {
                    public_id,
                    private_id,
                    room_id,
                    client_ip,
                };
                async move {
                    let start_time = Instant::now();
                    let timestamp = SystemTime::now()
                        .duration_since(UNIX_EPOCH)
                        .unwrap()
                        .as_secs();
                    let response = RoomHttpHandler::new(app_context, request_context)
                        .can_connect(query_params.username)
                        .await;
                    let processing_time_ns = start_time.elapsed().as_nanos();
                    tracing::info!(
                        task = "http_request",
                        http_method = "GET",
                        endpoint = "/rooms/<room_id>/can-connect?username=<username>",
                        client_ip = client_ip.unwrap_or(DEFAULT_CLIENT_IP).ip().to_string(),
                        processing_time_ms = processing_time_ns / 1000,
                        timestamp = timestamp,
                    );
                    Ok::<_, Infallible>(serde_json::to_string(&response).unwrap())
                }
            }
        })
        .with(warp::reply::with::headers(headers.clone()))
        .with(cors.clone());

    let is_host = warp::path("rooms")
        .and(warp::path::param::<String>())
        .and(warp::path("am-i-host"))
        .and(warp::header::<String>("Public-ID"))
        .and(warp::header::<String>("Private-ID"))
        .and(warp::addr::remote())
        .and_then({
            let app_context = app_context.clone();
            move |room_id, public_id, private_id, client_ip: Option<SocketAddr>| {
                let app_context = app_context.clone();
                let request_context = RequestContext {
                    public_id,
                    private_id,
                    room_id,
                    client_ip,
                };
                async move {
                    let start_time = Instant::now();
                    let timestamp = SystemTime::now()
                        .duration_since(UNIX_EPOCH)
                        .unwrap()
                        .as_secs();
                    let response = UsersHttpHandler::new(app_context, request_context)
                        .is_host()
                        .await;
                    let processing_time_ns = start_time.elapsed().as_nanos();
                    tracing::info!(
                        task = "http_request",
                        http_method = "GET",
                        endpoint = "/rooms/<room_id>/am-i-host",
                        client_ip = client_ip.unwrap_or(DEFAULT_CLIENT_IP).ip().to_string(),
                        processing_time_ms = processing_time_ns / 1000,
                        timestamp = timestamp,
                    );
                    Ok::<_, Infallible>(serde_json::to_string(&response).unwrap())
                }
            }
        })
        .with(warp::reply::with::headers(headers.clone()))
        .with(cors.clone());

    let submit_guess = warp::post()
        .and(warp::path("rooms"))
        .and(warp::path::param::<String>())
        .and(warp::path("submit-guess"))
        .and(warp::header::<String>("Public-ID"))
        .and(warp::header::<String>("Private-ID"))
        .and(warp::body::json())
        .and(warp::addr::remote())
        .and_then({
            let app_context = app_context.clone();
            move |room_id,
                  public_id,
                  private_id,
                  guess_json: HashMap<String, String>,
                  client_ip: Option<SocketAddr>| {
                let app_context = app_context.clone();
                let request_context = RequestContext {
                    public_id,
                    private_id,
                    room_id,
                    client_ip,
                };
                async move {
                    let start_time = Instant::now();
                    let timestamp = SystemTime::now()
                        .duration_since(UNIX_EPOCH)
                        .unwrap()
                        .as_secs();
                    let guess = LatLng {
                        lat: guess_json.get("lat").unwrap().parse().unwrap(),
                        lng: guess_json.get("lng").unwrap().parse().unwrap(),
                    };
                    let response = UsersHttpHandler::new(app_context, request_context)
                        .submit_guess(guess)
                        .await;
                    let processing_time_ns = start_time.elapsed().as_nanos();
                    tracing::info!(
                        task = "http_request",
                        http_method = "POST",
                        endpoint = "/rooms/<room_id>/submit-guess",
                        client_ip = client_ip.unwrap_or(DEFAULT_CLIENT_IP).ip().to_string(),
                        processing_time_ms = processing_time_ns / 1000,
                        timestamp = timestamp,
                    );
                    Ok::<_, Infallible>(serde_json::to_string(&response).unwrap())
                }
            }
        })
        .with(warp::reply::with::headers(headers.clone()))
        .with(cors.clone());

    let revoke_guess = warp::post()
        .and(warp::path("rooms"))
        .and(warp::path::param::<String>())
        .and(warp::path("revoke-guess"))
        .and(warp::header::<String>("Public-ID"))
        .and(warp::header::<String>("Private-ID"))
        .and(warp::addr::remote())
        .and_then({
            let app_context = app_context.clone();
            move |room_id, public_id, private_id, client_ip: Option<SocketAddr>| {
                let app_context = app_context.clone();
                let request_context = RequestContext {
                    public_id,
                    private_id,
                    room_id,
                    client_ip,
                };
                async move {
                    let start_time = Instant::now();
                    let timestamp = SystemTime::now()
                        .duration_since(UNIX_EPOCH)
                        .unwrap()
                        .as_secs();
                    let response = UsersHttpHandler::new(app_context, request_context)
                        .revoke_guess()
                        .await;
                    let processing_time_ns = start_time.elapsed().as_nanos();
                    tracing::info!(
                        task = "http_request",
                        http_method = "POST",
                        endpoint = "/rooms/<room_id>/revoke-guess",
                        client_ip = client_ip.unwrap_or(DEFAULT_CLIENT_IP).ip().to_string(),
                        processing_time_ms = processing_time_ns / 1000,
                        timestamp = timestamp,
                    );
                    Ok::<_, Infallible>(serde_json::to_string(&response).unwrap())
                }
            }
        })
        .with(warp::reply::with::headers(headers.clone()))
        .with(cors.clone());

    // TODO: this should be `POST` (?)
    let acquire_id = warp::path("auth")
        .and(warp::path("acquire-ids"))
        .and(warp::addr::remote())
        .and_then(move |client_ip: Option<SocketAddr>| async move {
            let start_time = Instant::now();
            let timestamp = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs();
            let response = AuthHttpHandler::new().acquire_id().await;
            let processing_time_ns = start_time.elapsed().as_nanos();
            tracing::info!(
                task = "http_request",
                http_method = "GET",
                endpoint = "/auth/acquire-ids",
                client_ip = client_ip.unwrap_or(DEFAULT_CLIENT_IP).ip().to_string(),
                processing_time_ms = processing_time_ns / 1000,
                timestamp = timestamp,
            );
            Ok::<_, Infallible>(serde_json::to_string(&response).unwrap())
        })
        .with(warp::reply::with::headers(headers.clone()))
        .with(cors.clone());

    let mute_user = warp::post()
        .and(warp::path("rooms"))
        .and(warp::path::param::<String>())
        .and(warp::path("users"))
        .and(warp::path::param::<String>())
        .and(warp::path("mute"))
        .and(warp::header::<String>("Public-ID"))
        .and(warp::header::<String>("Private-ID"))
        .and(warp::addr::remote())
        .and_then({
            let app_context = app_context.clone();
            move |room_id,
                  target_user_public_id,
                  public_id,
                  private_id,
                  client_ip: Option<SocketAddr>| {
                let app_context = app_context.clone();
                let request_context = RequestContext {
                    public_id,
                    private_id,
                    room_id,
                    client_ip,
                };
                async move {
                    let start_time = Instant::now();
                    let timestamp = SystemTime::now()
                        .duration_since(UNIX_EPOCH)
                        .unwrap()
                        .as_secs();
                    let response = UsersHttpHandler::new(app_context, request_context)
                        .mute(target_user_public_id)
                        .await;
                    let processing_time_ns = start_time.elapsed().as_nanos();
                    tracing::info!(
                        task = "http_request",
                        http_method = "POST",
                        endpoint = "/rooms/<room_id>/users/<user_id>/mute",
                        client_ip = client_ip.unwrap_or(DEFAULT_CLIENT_IP).ip().to_string(),
                        processing_time_ms = processing_time_ns / 1000,
                        timestamp = timestamp,
                    );
                    Ok::<_, Infallible>(serde_json::to_string(&response).unwrap())
                }
            }
        })
        .with(warp::reply::with::headers(headers.clone()))
        .with(cors.clone());

    let unmute_user = warp::post()
        .and(warp::path("rooms"))
        .and(warp::path::param::<String>())
        .and(warp::path("users"))
        .and(warp::path::param::<String>())
        .and(warp::path("unmute"))
        .and(warp::header::<String>("Public-ID"))
        .and(warp::header::<String>("Private-ID"))
        .and(warp::addr::remote())
        .and_then({
            let app_context = app_context.clone();
            move |room_id,
                  target_user_public_id,
                  public_id,
                  private_id,
                  client_ip: Option<SocketAddr>| {
                let app_context = app_context.clone();
                let request_context = RequestContext {
                    public_id,
                    private_id,
                    room_id,
                    client_ip,
                };
                async move {
                    let start_time = Instant::now();
                    let timestamp = SystemTime::now()
                        .duration_since(UNIX_EPOCH)
                        .unwrap()
                        .as_secs();
                    let response = UsersHttpHandler::new(app_context, request_context)
                        .unmute(target_user_public_id)
                        .await;
                    let processing_time_ns = start_time.elapsed().as_nanos();
                    tracing::info!(
                        task = "http_request",
                        http_method = "POST",
                        endpoint = "/rooms/<room_id>/users/<user_id>/unmute",
                        client_ip = client_ip.unwrap_or(DEFAULT_CLIENT_IP).ip().to_string(),
                        processing_time_ms = processing_time_ns / 1000,
                        timestamp = timestamp,
                    );
                    Ok::<_, Infallible>(serde_json::to_string(&response).unwrap())
                }
            }
        })
        .with(warp::reply::with::headers(headers.clone()))
        .with(cors.clone());

    let ban_user = warp::post()
        .and(warp::path("rooms"))
        .and(warp::path::param::<String>())
        .and(warp::path("users"))
        .and(warp::path::param::<String>())
        .and(warp::path("ban"))
        .and(warp::header::<String>("Public-ID"))
        .and(warp::header::<String>("Private-ID"))
        .and(warp::addr::remote())
        .and_then({
            let app_context = app_context.clone();
            move |room_id,
                  target_user_public_id,
                  public_id,
                  private_id,
                  client_ip: Option<SocketAddr>| {
                let app_context = app_context.clone();
                let request_context = RequestContext {
                    public_id,
                    private_id,
                    room_id,
                    client_ip,
                };
                async move {
                    let start_time = Instant::now();
                    let timestamp = SystemTime::now()
                        .duration_since(UNIX_EPOCH)
                        .unwrap()
                        .as_secs();
                    let response = UsersHttpHandler::new(app_context, request_context)
                        .ban(target_user_public_id)
                        .await;
                    let processing_time_ns = start_time.elapsed().as_nanos();
                    tracing::info!(
                        task = "http_request",
                        http_method = "POST",
                        endpoint = "/rooms/<room_id>/users/<user_id>/ban",
                        client_ip = client_ip.unwrap_or(DEFAULT_CLIENT_IP).ip().to_string(),
                        processing_time_ms = processing_time_ns / 1000,
                        timestamp = timestamp,
                    );
                    Ok::<_, Infallible>(serde_json::to_string(&response).unwrap())
                }
            }
        })
        .with(warp::reply::with::headers(headers.clone()))
        .with(cors.clone());

    let change_user_score = warp::post()
        .and(warp::path("rooms"))
        .and(warp::path::param::<String>())
        .and(warp::path("users"))
        .and(warp::path::param::<String>())
        .and(warp::path("change-score"))
        .and(warp::header::<String>("Public-ID"))
        .and(warp::header::<String>("Private-ID"))
        .and(warp::body::json())
        .and(warp::addr::remote())
        .and_then({
            let app_context = app_context.clone();
            move |room_id,
                  target_user_public_id,
                  public_id,
                  private_id,
                  request_body: HashMap<String, String>,
                  client_ip: Option<SocketAddr>| {
                let app_context = app_context.clone();
                let request_context = RequestContext {
                    public_id,
                    private_id,
                    room_id,
                    client_ip,
                };
                async move {
                    let start_time = Instant::now();
                    let timestamp = SystemTime::now()
                        .duration_since(UNIX_EPOCH)
                        .unwrap()
                        .as_secs();
                    let amount = request_body.get("amount").unwrap().parse::<i64>().unwrap();
                    let response = UsersHttpHandler::new(app_context, request_context)
                        .change_score(target_user_public_id, amount)
                        .await;
                    let processing_time_ns = start_time.elapsed().as_nanos();
                    tracing::info!(
                        task = "http_request",
                        http_method = "POST",
                        endpoint = "/rooms/<room_id>/users/<user_id>/change-score",
                        client_ip = client_ip.unwrap_or(DEFAULT_CLIENT_IP).ip().to_string(),
                        processing_time_ms = processing_time_ns / 1000,
                        timestamp = timestamp,
                    );
                    Ok::<_, Infallible>(serde_json::to_string(&response).unwrap())
                }
            }
        })
        .with(warp::reply::with::headers(headers.clone()))
        .with(cors.clone());

    let users_of_room = warp::path("rooms")
        .and(warp::path::param::<String>())
        .and(warp::path("users"))
        .and(warp::header::<String>("Public-ID"))
        .and(warp::header::<String>("Private-ID"))
        .and(warp::addr::remote())
        .and_then({
            let app_context = app_context.clone();
            move |room_id, public_id, private_id, client_ip: Option<SocketAddr>| {
                let request_context = RequestContext {
                    public_id,
                    private_id,
                    room_id,
                    client_ip,
                };
                let app_context = app_context.clone();
                async move {
                    let start_time = Instant::now();
                    let timestamp = SystemTime::now()
                        .duration_since(UNIX_EPOCH)
                        .unwrap()
                        .as_secs();
                    let response = RoomHttpHandler::new(app_context, request_context)
                        .users()
                        .await;
                    let processing_time_ns = start_time.elapsed().as_nanos();
                    tracing::info!(
                        task = "http_request",
                        http_method = "GET",
                        endpoint = "/rooms/<room_id>/users",
                        client_ip = client_ip.unwrap_or(DEFAULT_CLIENT_IP).ip().to_string(),
                        processing_time_ms = processing_time_ns / 1000,
                        timestamp = timestamp,
                    );
                    Ok::<_, Infallible>(serde_json::to_string(&response).unwrap())
                }
            }
        })
        .with(warp::reply::with::headers(headers.clone()))
        .with(cors.clone());

    let messages_of_room = warp::path("rooms")
        .and(warp::path::param::<String>())
        .and(warp::path("messages"))
        .and(warp::header::<String>("Public-ID"))
        .and(warp::header::<String>("Private-ID"))
        .and(warp::addr::remote())
        .and_then({
            let app_context = app_context.clone();
            move |room_id, public_id, private_id, client_ip: Option<SocketAddr>| {
                let app_context = app_context.clone();
                let request_context = RequestContext {
                    public_id,
                    private_id,
                    room_id,
                    client_ip,
                };
                async move {
                    let start_time = Instant::now();
                    let timestamp = SystemTime::now()
                        .duration_since(UNIX_EPOCH)
                        .unwrap()
                        .as_secs();
                    let response = RoomHttpHandler::new(app_context, request_context)
                        .messages()
                        .await;
                    let processing_time_ns = start_time.elapsed().as_nanos();
                    tracing::info!(
                        task = "http_request",
                        http_method = "GET",
                        endpoint = "/rooms/<room_id>/messages",
                        client_ip = client_ip.unwrap_or(DEFAULT_CLIENT_IP).ip().to_string(),
                        processing_time_ms = processing_time_ns / 1000,
                        timestamp = timestamp,
                    );
                    Ok::<_, Infallible>(serde_json::to_string(&response).unwrap())
                }
            }
        })
        .with(warp::reply::with::headers(headers.clone()))
        .with(cors.clone());

    let create_room = warp::post()
        .and(warp::path("rooms"))
        .and(warp::header::<String>("Public-ID"))
        .and(warp::header::<String>("Private-ID"))
        .and(warp::addr::remote())
        .and_then({
            let app_context = app_context.clone();
            move |_public_id, _private_id, client_ip: Option<SocketAddr>| {
                let app_context = app_context.clone();
                async move {
                    let start_time = Instant::now();
                    let timestamp = SystemTime::now()
                        .duration_since(UNIX_EPOCH)
                        .unwrap()
                        .as_secs();
                    let response = CreateRoomHttpHandler::new(app_context).create().await;
                    let processing_time_ns = start_time.elapsed().as_nanos();
                    tracing::info!(
                        task = "http_request",
                        http_method = "POST",
                        endpoint = "/rooms",
                        client_ip = client_ip.unwrap_or(DEFAULT_CLIENT_IP).ip().to_string(),
                        processing_time_ms = processing_time_ns / 1000,
                        timestamp = timestamp,
                    );
                    Ok::<_, Infallible>(serde_json::to_string(&response).unwrap())
                }
            }
        })
        .with(warp::reply::with::headers(headers.clone()))
        .with(cors.clone());

    let healthcheck = warp::path("health")
        .and(warp::path("check"))
        .and(warp::addr::remote())
        .and_then(move |client_ip: Option<SocketAddr>| async move {
            let start_time = Instant::now();
            let timestamp = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs();
            let response = HealthHttpHandler::new().healthcheck().await;
            let processing_time_ns = start_time.elapsed().as_nanos();
            tracing::info!(
                task = "http_request",
                http_method = "GET",
                endpoint = "/health/check",
                client_ip = client_ip.unwrap_or(DEFAULT_CLIENT_IP).ip().to_string(),
                processing_time_ms = processing_time_ns / 1000,
                timestamp = timestamp,
            );
            Ok::<_, Infallible>(serde_json::to_string(&response).unwrap())
        })
        .with(warp::reply::with::headers(headers.clone()))
        .with(cors.clone());

    let routes = rooms
        .or(can_connect_to_room)
        .or(is_host)
        .or(submit_guess)
        .or(revoke_guess)
        .or(acquire_id)
        .or(mute_user)
        .or(unmute_user)
        .or(ban_user)
        .or(change_user_score)
        .or(users_of_room)
        .or(messages_of_room)
        .or(create_room)
        .or(healthcheck)
        .with(cors);

    warp::serve(routes).run(([0, 0, 0, 0], 3030)).await;
}
