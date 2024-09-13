use crate::app_context::{AppContext, RequestContext};
use crate::auth::passcode;
use crate::http::config::{CORS_POLICY, RESPONSE_HEADERS};
use crate::logging::consts::DEFAULT_CLIENT_IP;
use crate::storage::rooms::HashMapRoomsStorage;
use crate::users::handlers::UsersHttpHandler;
use std::collections::HashMap;
use std::convert::Infallible;
use std::net::SocketAddr;
use std::time::{SystemTime, UNIX_EPOCH};
use tokio::time::Instant;
use warp::reply::Reply;
use warp::{http::StatusCode, Filter, Rejection};

pub fn mute_user(
    app_context: &AppContext<HashMapRoomsStorage>,
) -> impl Filter<Extract = (impl Reply,), Error = Rejection> + Clone {
    warp::post()
        .and(warp::path("rooms"))
        .and(warp::path::param::<String>())
        .and(warp::path("users"))
        .and(warp::path::param::<String>())
        .and(warp::path("mute"))
        .and(warp::header::<String>("Passcode"))
        .and(warp::addr::remote())
        .and_then({
            let app_context = app_context.clone();
            move |room_id,
                  target_user_public_id,
                  passcode: String,
                  client_ip: Option<SocketAddr>| {
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
                    let response = UsersHttpHandler::new(app_context, &request_context)
                        .mute(target_user_public_id)
                        .await;
                    let processing_time_ns = start_time.elapsed().as_nanos();
                    tracing::info!(
                        task = "http_request",
                        http_method = "POST",
                        private_id = request_context.private_id,
                        endpoint = "/rooms/<room_id>/users/<user_id>/mute",
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

pub fn unmute_user(
    app_context: &AppContext<HashMapRoomsStorage>,
) -> impl Filter<Extract = (impl Reply,), Error = Rejection> + Clone {
    warp::post()
        .and(warp::path("rooms"))
        .and(warp::path::param::<String>())
        .and(warp::path("users"))
        .and(warp::path::param::<String>())
        .and(warp::path("unmute"))
        .and(warp::header::<String>("Passcode"))
        .and(warp::addr::remote())
        .and_then({
            let app_context = app_context.clone();
            move |room_id,
                  target_user_public_id,
                  passcode: String,
                  client_ip: Option<SocketAddr>| {
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
                    let response = UsersHttpHandler::new(app_context, &request_context)
                        .unmute(target_user_public_id)
                        .await;
                    let processing_time_ns = start_time.elapsed().as_nanos();
                    tracing::info!(
                        task = "http_request",
                        http_method = "POST",
                        endpoint = "/rooms/<room_id>/users/<user_id>/unmute",
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

pub fn ban_user(
    app_context: &AppContext<HashMapRoomsStorage>,
) -> impl Filter<Extract = (impl Reply,), Error = Rejection> + Clone {
    warp::post()
        .and(warp::path("rooms"))
        .and(warp::path::param::<String>())
        .and(warp::path("users"))
        .and(warp::path::param::<String>())
        .and(warp::path("ban"))
        .and(warp::header::<String>("Passcode"))
        .and(warp::addr::remote())
        .and_then({
            let app_context = app_context.clone();
            move |room_id,
                  target_user_public_id,
                  passcode: String,
                  client_ip: Option<SocketAddr>| {
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
                    let response = UsersHttpHandler::new(app_context, &request_context)
                        .ban(target_user_public_id)
                        .await;
                    let processing_time_ns = start_time.elapsed().as_nanos();
                    tracing::info!(
                        task = "http_request",
                        http_method = "POST",
                        endpoint = "/rooms/<room_id>/users/<user_id>/ban",
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

pub fn change_user_score(
    app_context: &AppContext<HashMapRoomsStorage>,
) -> impl Filter<Extract = (impl Reply,), Error = Rejection> + Clone {
    warp::post()
        .and(warp::path("rooms"))
        .and(warp::path::param::<String>())
        .and(warp::path("users"))
        .and(warp::path::param::<String>())
        .and(warp::path("change-score"))
        .and(warp::header::<String>("Passcode"))
        .and(warp::body::json())
        .and(warp::addr::remote())
        .and_then({
            let app_context = app_context.clone();
            move |room_id,
                  target_user_public_id,
                  passcode: String,
                  request_body: HashMap<String, String>,
                  client_ip: Option<SocketAddr>| {
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
                    let amount = request_body.get("amount").unwrap().parse::<i64>().unwrap();
                    let response = UsersHttpHandler::new(app_context, &request_context)
                        .change_score(target_user_public_id, amount)
                        .await;
                    let processing_time_ns = start_time.elapsed().as_nanos();
                    tracing::info!(
                        task = "http_request",
                        http_method = "POST",
                        endpoint = "/rooms/<room_id>/users/<user_id>/change-score",
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
