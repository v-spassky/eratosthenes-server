use crate::app_context::{AppContext, RequestContext};
use crate::auth::handlers::AuthHttpHandler;
use crate::health::handlers::HealthHttpHandler;
use crate::query_params::{UserIdsQueryParams, UsernameQueryParam};
use crate::rooms::handlers::http::{CreateRoomHttpHandler, RoomHttpHandler};
use crate::rooms::handlers::ws::RoomWsHandler;
use crate::storage::rooms::HashMapRoomsStorage;
use crate::users::handlers::UsersHttpHandler;
use map_locations::models::LatLng;
use std::collections::HashMap;
use std::convert::Infallible;
use warp::http::header::{HeaderMap, HeaderValue};
use warp::{hyper::Method, ws::Ws, Filter};

mod app_context;
mod auth;
mod health;
mod map_locations;
mod query_params;
mod rooms;
mod storage;
mod users;

#[tokio::main]
async fn main() {
    let app_context = AppContext::<HashMapRoomsStorage>::default();

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
        .map({
            let app_context = app_context.clone();
            move |ws: Ws, room_id, query_params: UserIdsQueryParams| {
                let app_context = app_context.clone();
                let request_context = RequestContext {
                    public_id: query_params.public_id,
                    private_id: query_params.private_id,
                    room_id,
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
        .and_then({
            let app_context = app_context.clone();
            move |room_id, query_params: UsernameQueryParam, public_id, private_id| {
                let app_context = app_context.clone();
                let request_context = RequestContext {
                    public_id,
                    private_id,
                    room_id,
                };
                async move {
                    let response = RoomHttpHandler::new(app_context, request_context)
                        .can_connect(query_params.username)
                        .await;
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
        .and_then({
            let app_context = app_context.clone();
            move |room_id, public_id, private_id| {
                let app_context = app_context.clone();
                let request_context = RequestContext {
                    public_id,
                    private_id,
                    room_id,
                };
                async move {
                    let response = UsersHttpHandler::new(app_context, request_context)
                        .is_host()
                        .await;
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
        .and_then({
            let app_context = app_context.clone();
            move |room_id, public_id, private_id, guess_json: HashMap<String, String>| {
                let app_context = app_context.clone();
                let request_context = RequestContext {
                    public_id,
                    private_id,
                    room_id,
                };
                async move {
                    let guess = LatLng {
                        lat: guess_json.get("lat").unwrap().parse().unwrap(),
                        lng: guess_json.get("lng").unwrap().parse().unwrap(),
                    };
                    let response = UsersHttpHandler::new(app_context, request_context)
                        .submit_guess(guess)
                        .await;
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
        .and_then({
            let app_context = app_context.clone();
            move |room_id, public_id, private_id| {
                let app_context = app_context.clone();
                let request_context = RequestContext {
                    public_id,
                    private_id,
                    room_id,
                };
                async move {
                    let response = UsersHttpHandler::new(app_context, request_context)
                        .revoke_guess()
                        .await;
                    Ok::<_, Infallible>(serde_json::to_string(&response).unwrap())
                }
            }
        })
        .with(warp::reply::with::headers(headers.clone()))
        .with(cors.clone());

    // TODO: this should be `POST` (?)
    let acquire_id = warp::path("auth")
        .and(warp::path("acquire-ids"))
        .and_then(move || async move {
            let response = AuthHttpHandler::new().acquire_id().await;
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
        .and_then({
            let app_context = app_context.clone();
            move |room_id, target_user_public_id, public_id, private_id| {
                let app_context = app_context.clone();
                let request_context = RequestContext {
                    public_id,
                    private_id,
                    room_id,
                };
                async move {
                    let response = UsersHttpHandler::new(app_context, request_context)
                        .mute(target_user_public_id)
                        .await;
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
        .and_then({
            let app_context = app_context.clone();
            move |room_id, target_user_public_id, public_id, private_id| {
                let app_context = app_context.clone();
                let request_context = RequestContext {
                    public_id,
                    private_id,
                    room_id,
                };
                async move {
                    let response = UsersHttpHandler::new(app_context, request_context)
                        .unmute(target_user_public_id)
                        .await;
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
        .and_then({
            let app_context = app_context.clone();
            move |room_id, target_user_public_id, public_id, private_id| {
                let app_context = app_context.clone();
                let request_context = RequestContext {
                    public_id,
                    private_id,
                    room_id,
                };
                async move {
                    let response = UsersHttpHandler::new(app_context, request_context)
                        .ban(target_user_public_id)
                        .await;
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
        .and_then({
            let app_context = app_context.clone();
            move |room_id,
                  target_user_public_id,
                  public_id,
                  private_id,
                  request_body: HashMap<String, String>| {
                let app_context = app_context.clone();
                let request_context = RequestContext {
                    public_id,
                    private_id,
                    room_id,
                };
                async move {
                    let amount = request_body.get("amount").unwrap().parse::<i64>().unwrap();
                    let response = UsersHttpHandler::new(app_context, request_context)
                        .change_score(target_user_public_id, amount)
                        .await;
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
        .and_then({
            let app_context = app_context.clone();
            move |room_id, public_id, private_id| {
                let request_context = RequestContext {
                    public_id,
                    private_id,
                    room_id,
                };
                let app_context = app_context.clone();
                async move {
                    let response = RoomHttpHandler::new(app_context, request_context)
                        .users()
                        .await;
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
        .and_then({
            let app_context = app_context.clone();
            move |room_id, public_id, private_id| {
                let app_context = app_context.clone();
                let request_context = RequestContext {
                    public_id,
                    private_id,
                    room_id,
                };
                async move {
                    let response = RoomHttpHandler::new(app_context, request_context)
                        .messages()
                        .await;
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
        .and_then({
            let app_context = app_context.clone();
            move |_public_id, _private_id| {
                let app_context = app_context.clone();
                async move {
                    let response = CreateRoomHttpHandler::new(app_context).create().await;
                    Ok::<_, Infallible>(serde_json::to_string(&response).unwrap())
                }
            }
        })
        .with(warp::reply::with::headers(headers.clone()))
        .with(cors.clone());

    let healthcheck = warp::path("health")
        .and(warp::path("check"))
        .and_then(move || async move {
            let response = HealthHttpHandler::new().healthcheck().await;
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
