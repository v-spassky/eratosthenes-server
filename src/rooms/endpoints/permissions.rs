use crate::app_context::{AppContext, RequestContext};
use crate::auth::passcode;
use crate::http::config::{CORS_POLICY, RESPONSE_HEADERS};
use crate::http::query_params::UsernameQueryParam;
use crate::logging::consts::DEFAULT_CLIENT_IP;
use crate::rooms::handlers::http::RoomHttpHandler;
use crate::storage::rooms::HashMapRoomsStorage;
use crate::users::handlers::UsersHttpHandler;
use std::convert::Infallible;
use std::net::SocketAddr;
use std::time::{SystemTime, UNIX_EPOCH};
use tokio::time::Instant;
use warp::reply::Reply;
use warp::{http::StatusCode, Filter, Rejection};

pub fn can_connect_to_room(
    app_context: &AppContext<HashMapRoomsStorage>,
) -> impl Filter<Extract = (impl Reply,), Error = Rejection> + Clone {
    warp::path("rooms")
        .and(warp::path::param::<String>())
        .and(warp::path("can-connect"))
        .and(warp::query::<UsernameQueryParam>())
        .and(warp::header::<String>("Passcode"))
        .and(warp::addr::remote())
        .and_then({
            let app_context = app_context.clone();
            move |room_id,
                  query_params: UsernameQueryParam,
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
                    let response = RoomHttpHandler::new(app_context, &request_context)
                        .can_connect(query_params.username)
                        .await;
                    let processing_time_ns = start_time.elapsed().as_nanos();
                    tracing::info!(
                        task = "http_request",
                        http_method = "GET",
                        endpoint = "/rooms/<room_id>/can-connect?username=<username>",
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
        .with(warp::reply::with::headers(
            RESPONSE_HEADERS
                .get()
                .expect("`RESPONSE_HEADERS` was not initialized.")
                .clone(),
        ))
        .with(
            CORS_POLICY
                .get()
                .expect("`CORS_POLICY` was not initialized.")
                .clone(),
        )
}

pub fn is_host(
    app_context: &AppContext<HashMapRoomsStorage>,
) -> impl Filter<Extract = (impl Reply,), Error = Rejection> + Clone {
    warp::path("rooms")
        .and(warp::path::param::<String>())
        .and(warp::path("am-i-host"))
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
                    let response = UsersHttpHandler::new(app_context, &request_context)
                        .is_host()
                        .await;
                    let processing_time_ns = start_time.elapsed().as_nanos();
                    tracing::info!(
                        task = "http_request",
                        http_method = "GET",
                        endpoint = "/rooms/<room_id>/am-i-host",
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
        .with(warp::reply::with::headers(
            RESPONSE_HEADERS
                .get()
                .expect("`RESPONSE_HEADERS` was not initialized.")
                .clone(),
        ))
        .with(
            CORS_POLICY
                .get()
                .expect("`CORS_POLICY` was not initialized.")
                .clone(),
        )
}
