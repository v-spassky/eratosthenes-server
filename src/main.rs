use std::collections::HashMap;

use crate::app_context::AppContext;
use crate::auth::handlers::AuthHttpHandler;
use crate::health::handlers::HealthHttpHandler;
use crate::query_params::{UserIdQueryParam, UsernameQueryParam};
use crate::rooms::handlers::http::{CreateRoomHttpHandler, RoomHttpHandler};
use crate::rooms::handlers::ws::RoomWsHandler;
use crate::storage::rooms::HashMapRoomsStorage;
use crate::users::handlers::UsersHttpHandler;
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
        ])
        .allow_methods(&[Method::POST, Method::GET, Method::OPTIONS])
        .build();

    let rooms = warp::path("rooms")
        .and(warp::ws())
        .and(warp::path::param::<String>())
        .and(warp::query::<UserIdQueryParam>())
        .map({
            let app_context = app_context.clone();
            move |ws: Ws, room_id, UserIdQueryParam { user_id }: UserIdQueryParam| {
                let app_context = app_context.clone();
                ws.on_upgrade(|socket| async {
                    RoomWsHandler::new(app_context, socket, room_id, user_id)
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
        .and(warp::query::<UserIdQueryParam>())
        .and(warp::query::<UsernameQueryParam>())
        .and_then({
            let app_context = app_context.clone();
            move |room_id: String,
                  UserIdQueryParam { user_id }: UserIdQueryParam,
                  UsernameQueryParam { username }: UsernameQueryParam| {
                let app_context = app_context.clone();
                async move {
                    RoomHttpHandler::new(app_context, room_id, user_id)
                        .can_connect(username)
                        .await
                }
            }
        })
        .with(cors.clone());

    let users_of_room = warp::path("rooms")
        .and(warp::path::param::<String>())
        .and(warp::path("users"))
        .and(warp::query::<UserIdQueryParam>())
        .and_then({
            let app_context = app_context.clone();
            move |room_id: String, UserIdQueryParam { user_id }: UserIdQueryParam| {
                let app_context = app_context.clone();
                async move {
                    RoomHttpHandler::new(app_context, room_id, user_id)
                        .users()
                        .await
                }
            }
        })
        .with(cors.clone());

    let messages_of_room = warp::path("rooms")
        .and(warp::path::param::<String>())
        .and(warp::path("messages"))
        .and(warp::query::<UserIdQueryParam>())
        .and_then({
            let app_context = app_context.clone();
            move |room_id: String, UserIdQueryParam { user_id }: UserIdQueryParam| {
                let app_context = app_context.clone();
                async move {
                    RoomHttpHandler::new(app_context, room_id, user_id)
                        .messages()
                        .await
                }
            }
        })
        .with(cors.clone());

    let is_host = warp::path("rooms")
        .and(warp::path::param::<String>())
        .and(warp::path("am-i-host"))
        .and(warp::query::<UserIdQueryParam>())
        .and_then({
            let app_context = app_context.clone();
            move |room_id: String, UserIdQueryParam { user_id }: UserIdQueryParam| {
                let app_context = app_context.clone();
                async move {
                    UsersHttpHandler::new(app_context, room_id, user_id)
                        .is_host()
                        .await
                }
            }
        })
        .with(cors.clone());

    let submit_guess = warp::post()
        .and(warp::path("rooms"))
        .and(warp::path::param::<String>())
        .and(warp::path("submit-guess"))
        .and(warp::query::<UserIdQueryParam>())
        .and(warp::body::json())
        .and_then({
            let app_context = app_context.clone();
            move |room_id: String,
                  UserIdQueryParam { user_id }: UserIdQueryParam,
                  guess_json: HashMap<String, String>| {
                let app_context = app_context.clone();
                async move {
                    UsersHttpHandler::new(app_context, room_id, user_id)
                        .submit_guess(guess_json)
                        .await
                }
            }
        })
        .with(cors.clone());

    let revoke_guess = warp::post()
        .and(warp::path("rooms"))
        .and(warp::path::param::<String>())
        .and(warp::path("revoke-guess"))
        .and(warp::query::<UserIdQueryParam>())
        .and_then({
            let app_context = app_context.clone();
            move |room_id: String, UserIdQueryParam { user_id }: UserIdQueryParam| {
                let app_context = app_context.clone();
                async move {
                    UsersHttpHandler::new(app_context, room_id, user_id)
                        .revoke_guess()
                        .await
                }
            }
        })
        .with(cors.clone());

    // TODO: this should be `POST` (?)
    let acquire_id = warp::path("auth")
        .and(warp::path("acquire-id"))
        .and_then(move || async move { AuthHttpHandler::new().acquire_id().await })
        .with(cors.clone());

    let mute_user = warp::post()
        .and(warp::path("rooms"))
        .and(warp::path::param::<String>())
        .and(warp::path("users"))
        .and(warp::path::param::<String>())
        .and(warp::path("mute"))
        .and(warp::query::<UserIdQueryParam>())
        .and_then({
            let app_context = app_context.clone();
            move |room_id: String,
                  target_username: String,
                  UserIdQueryParam { user_id }: UserIdQueryParam| {
                let app_context = app_context.clone();
                async move {
                    UsersHttpHandler::new(app_context, room_id, user_id)
                        .mute(target_username)
                        .await
                }
            }
        })
        .with(cors.clone());

    let unmute_user = warp::post()
        .and(warp::path("rooms"))
        .and(warp::path::param::<String>())
        .and(warp::path("users"))
        .and(warp::path::param::<String>())
        .and(warp::path("unmute"))
        .and(warp::query::<UserIdQueryParam>())
        .and_then({
            let app_context = app_context.clone();
            move |room_id: String,
                  target_username: String,
                  UserIdQueryParam { user_id }: UserIdQueryParam| {
                let app_context = app_context.clone();
                async move {
                    UsersHttpHandler::new(app_context, room_id, user_id)
                        .unmute(target_username)
                        .await
                }
            }
        })
        .with(cors.clone());

    let ban_user = warp::post()
        .and(warp::path("rooms"))
        .and(warp::path::param::<String>())
        .and(warp::path("users"))
        .and(warp::path::param::<String>())
        .and(warp::path("user"))
        .and(warp::query::<UserIdQueryParam>())
        .and_then({
            let app_context = app_context.clone();
            move |room_id: String,
                  target_username: String,
                  UserIdQueryParam { user_id }: UserIdQueryParam| {
                let app_context = app_context.clone();
                async move {
                    UsersHttpHandler::new(app_context, room_id, user_id)
                        .ban(target_username)
                        .await
                }
            }
        })
        .with(cors.clone());

    let change_user_score = warp::post()
        .and(warp::path("rooms"))
        .and(warp::path::param::<String>())
        .and(warp::path("users"))
        .and(warp::path::param::<String>())
        .and(warp::path("change-score"))
        .and(warp::query::<UserIdQueryParam>())
        .and(warp::body::json())
        .and_then({
            let app_context = app_context.clone();
            move |room_id: String,
                  target_username: String,
                  UserIdQueryParam { user_id }: UserIdQueryParam,
                  request_body: HashMap<String, String>| {
                let app_context = app_context.clone();
                async move {
                    UsersHttpHandler::new(app_context, room_id, user_id)
                        .change_score(target_username, request_body)
                        .await
                }
            }
        })
        .with(cors.clone());

    let create_room = warp::post()
        .and(warp::path("rooms"))
        .and(warp::query::<UserIdQueryParam>())
        .and_then({
            let app_context = app_context.clone();
            move |_query_params: UserIdQueryParam| {
                let app_context = app_context.clone();
                async move { CreateRoomHttpHandler::new(app_context).create().await }
            }
        })
        .with(cors.clone());

    let healthcheck = warp::path("health")
        .and(warp::path("check"))
        .and_then(move || async move { HealthHttpHandler::new().healthcheck().await })
        .with(cors.clone());

    let routes = rooms
        .or(can_connect_to_room)
        .or(is_host)
        .or(users_of_room)
        .or(messages_of_room)
        .or(submit_guess)
        .or(revoke_guess)
        .or(acquire_id)
        .or(mute_user)
        .or(unmute_user)
        .or(ban_user)
        .or(change_user_score)
        .or(create_room)
        .or(healthcheck)
        .with(cors);

    warp::serve(routes).run(([0, 0, 0, 0], 3030)).await;
}
