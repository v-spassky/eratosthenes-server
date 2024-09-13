use crate::app_context::{AppContext, RequestContext};
use crate::auth::passcode;
use crate::http::config::{CORS_POLICY, RESPONSE_HEADERS};
use crate::logging::consts::DEFAULT_CLIENT_IP;
use crate::rooms::handlers::http::{CreateRoomHttpHandler, RoomHttpHandler};
use crate::storage::rooms::HashMapRoomsStorage;
use std::convert::Infallible;
use std::net::SocketAddr;
use std::time::{SystemTime, UNIX_EPOCH};
use tokio::time::Instant;
use warp::reply::Reply;
use warp::{http::StatusCode, Filter, Rejection};

pub fn users(
    app_context: &AppContext<HashMapRoomsStorage>,
) -> impl Filter<Extract = (impl Reply,), Error = Rejection> + Clone {
    warp::path("rooms")
        .and(warp::path::param::<String>())
        .and(warp::path("users"))
        .and(warp::header::<String>("Passcode"))
        .and(warp::addr::remote())
        .and_then({
            let app_context = app_context.clone();
            move |room_id, passcode: String, client_ip: Option<SocketAddr>| {
                let app_context = app_context.clone();
                async move {
                    let start_time = Instant::now();
                    let timestamp = SystemTime::now()
                        .duration_since(UNIX_EPOCH)
                        .unwrap()
                        .as_secs();
                    let jwt_payload = match passcode::decode(&passcode) {
                        Ok(payload) => payload,
                        Err(_) => {
                            return Ok(warp::reply::with_status(
                                "".to_string(),
                                StatusCode::UNAUTHORIZED,
                            ))
                        }
                    };
                    let request_context = RequestContext {
                        public_id: jwt_payload.public_id,
                        private_id: jwt_payload.private_id,
                        room_id,
                        client_ip,
                    };
                    let response = RoomHttpHandler::new(app_context, &request_context)
                        .users()
                        .await;
                    let processing_time_ns = start_time.elapsed().as_nanos();
                    tracing::info!(
                        task = "http_request",
                        http_method = "GET",
                        endpoint = "/rooms/<room_id>/users",
                        private_id = request_context.private_id,
                        client_ip = client_ip.unwrap_or(DEFAULT_CLIENT_IP).ip().to_string(),
                        processing_time_ms = processing_time_ns / 1000,
                        timestamp,
                    );
                    Ok::<_, Infallible>(warp::reply::with_status(
                        serde_json::to_string(&response).unwrap(),
                        StatusCode::OK,
                    ))
                }
            }
        })
        .with(warp::reply::with::headers(RESPONSE_HEADERS.clone()))
        .with(CORS_POLICY.clone())
}

pub fn messages(
    app_context: &AppContext<HashMapRoomsStorage>,
) -> impl Filter<Extract = (impl Reply,), Error = Rejection> + Clone {
    warp::path("rooms")
        .and(warp::path::param::<String>())
        .and(warp::path("messages"))
        .and(warp::header::<String>("Passcode"))
        .and(warp::addr::remote())
        .and_then({
            let app_context = app_context.clone();
            move |room_id, passcode: String, client_ip: Option<SocketAddr>| {
                let app_context = app_context.clone();
                async move {
                    let start_time = Instant::now();
                    let timestamp = SystemTime::now()
                        .duration_since(UNIX_EPOCH)
                        .unwrap()
                        .as_secs();
                    let jwt_payload = match passcode::decode(&passcode) {
                        Ok(payload) => payload,
                        Err(_) => {
                            return Ok(warp::reply::with_status(
                                "".to_string(),
                                StatusCode::UNAUTHORIZED,
                            ))
                        }
                    };
                    let request_context = RequestContext {
                        public_id: jwt_payload.public_id,
                        private_id: jwt_payload.private_id,
                        room_id,
                        client_ip,
                    };
                    let response = RoomHttpHandler::new(app_context, &request_context)
                        .messages()
                        .await;
                    let processing_time_ns = start_time.elapsed().as_nanos();
                    tracing::info!(
                        task = "http_request",
                        http_method = "GET",
                        endpoint = "/rooms/<room_id>/messages",
                        private_id = request_context.private_id,
                        client_ip = client_ip.unwrap_or(DEFAULT_CLIENT_IP).ip().to_string(),
                        processing_time_ms = processing_time_ns / 1000,
                        timestamp,
                    );
                    Ok::<_, Infallible>(warp::reply::with_status(
                        serde_json::to_string(&response).unwrap(),
                        StatusCode::OK,
                    ))
                }
            }
        })
        .with(warp::reply::with::headers(RESPONSE_HEADERS.clone()))
        .with(CORS_POLICY.clone())
}

pub fn create(
    app_context: &AppContext<HashMapRoomsStorage>,
) -> impl Filter<Extract = (impl Reply,), Error = Rejection> + Clone {
    warp::post()
        .and(warp::path("rooms"))
        .and(warp::header::<String>("Passcode"))
        .and(warp::addr::remote())
        .and_then({
            let app_context = app_context.clone();
            move |passcode: String, client_ip: Option<SocketAddr>| {
                let app_context = app_context.clone();
                async move {
                    let start_time = Instant::now();
                    let timestamp = SystemTime::now()
                        .duration_since(UNIX_EPOCH)
                        .unwrap()
                        .as_secs();
                    let jwt_payload = match passcode::decode(&passcode) {
                        Ok(payload) => payload,
                        Err(_) => {
                            return Ok(warp::reply::with_status(
                                "".to_string(),
                                StatusCode::UNAUTHORIZED,
                            ))
                        }
                    };
                    let response = CreateRoomHttpHandler::new(app_context).create().await;
                    let processing_time_ns = start_time.elapsed().as_nanos();
                    tracing::info!(
                        task = "http_request",
                        http_method = "POST",
                        endpoint = "/rooms",
                        private_id = jwt_payload.private_id,
                        client_ip = client_ip.unwrap_or(DEFAULT_CLIENT_IP).ip().to_string(),
                        processing_time_ms = processing_time_ns / 1000,
                        timestamp,
                    );
                    Ok::<_, Infallible>(warp::reply::with_status(
                        serde_json::to_string(&response).unwrap(),
                        StatusCode::OK,
                    ))
                }
            }
        })
        .with(warp::reply::with::headers(RESPONSE_HEADERS.clone()))
        .with(CORS_POLICY.clone())
}
