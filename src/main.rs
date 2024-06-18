use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use warp::{hyper::Method, Filter};

mod handlers;
mod map_locations;
mod message_types;
mod models;
mod storage;
mod user_descriptions;
mod user_id;

#[derive(Serialize, Deserialize)]
struct UserIdQueryParam {
    user_id: String,
}

#[derive(Serialize, Deserialize)]
struct UsernameQueryParam {
    username: String,
}

#[tokio::main]
async fn main() {
    let clients_sockets = storage::ClientSockets::default();
    let rooms = storage::Rooms::default();

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

    let chat = warp::path("chat")
        .and(warp::ws())
        .and(warp::path::param::<String>())
        .and(warp::query::<UserIdQueryParam>())
        .map({
            let clients_sockets = clients_sockets.clone();
            let rooms = rooms.clone();
            move |ws: warp::ws::Ws, room_id, UserIdQueryParam { user_id }: UserIdQueryParam| {
                let clients_sockets = clients_sockets.clone();
                let rooms = rooms.clone();
                ws.on_upgrade(|socket| {
                    handlers::user_connected(socket, clients_sockets, room_id, rooms, user_id)
                })
            }
        })
        .with(cors.clone());

    let can_connect = warp::path("can-connect")
        .and(warp::path::param::<String>())
        .and(warp::query::<UserIdQueryParam>())
        .and(warp::query::<UsernameQueryParam>())
        .and_then({
            let rooms = rooms.clone();
            move |room_id: String,
                  UserIdQueryParam { user_id }: UserIdQueryParam,
                  UsernameQueryParam { username }: UsernameQueryParam| {
                let rooms = rooms.clone();
                async move {
                    handlers::check_if_user_can_connect(rooms, room_id, user_id, username).await
                }
            }
        })
        .with(cors.clone());

    let is_host = warp::path("is-host")
        .and(warp::path::param::<String>())
        .and(warp::query::<UserIdQueryParam>())
        .and_then({
            let rooms = rooms.clone();
            move |room_id: String, UserIdQueryParam { user_id }: UserIdQueryParam| {
                let rooms = rooms.clone();
                async move { handlers::check_if_user_is_host(rooms, room_id, user_id).await }
            }
        })
        .with(cors.clone());

    let create_room = warp::post()
        .and(warp::path("create-room"))
        .and(warp::query::<UserIdQueryParam>())
        .and_then({
            let rooms = rooms.clone();
            move |UserIdQueryParam { user_id }: UserIdQueryParam| {
                let rooms = rooms.clone();
                async move { handlers::create_room(rooms, user_id).await }
            }
        })
        .with(cors.clone());

    let users_of_room = warp::path("users-of-room")
        .and(warp::path::param::<String>())
        .and(warp::query::<UserIdQueryParam>())
        .and_then({
            let rooms = rooms.clone();
            move |room_id: String, UserIdQueryParam { user_id }: UserIdQueryParam| {
                let rooms = rooms.clone();
                async move { handlers::get_users_of_room(rooms, room_id, user_id).await }
            }
        })
        .with(cors.clone());

    let messages_of_room = warp::path("messages-of-room")
        .and(warp::path::param::<String>())
        .and(warp::query::<UserIdQueryParam>())
        .and_then({
            let rooms = rooms.clone();
            move |room_id: String, UserIdQueryParam { user_id }: UserIdQueryParam| {
                let rooms = rooms.clone();
                async move { handlers::get_messages_of_room(rooms, room_id, user_id).await }
            }
        })
        .with(cors.clone());

    let submit_guess = warp::post()
        .and(warp::path("submit-guess"))
        .and(warp::path::param::<String>())
        .and(warp::query::<UserIdQueryParam>())
        .and(warp::body::json())
        .and_then({
            let rooms = rooms.clone();
            let clients_sockets = clients_sockets.clone();
            move |room_id: String,
                  UserIdQueryParam { user_id }: UserIdQueryParam,
                  guess_json: HashMap<String, String>| {
                let rooms = rooms.clone();
                let clients_sockets = clients_sockets.clone();
                async move {
                    handlers::submit_guess(rooms, room_id, user_id, guess_json, clients_sockets)
                        .await
                }
            }
        })
        .with(cors.clone());

    let revoke_guess = warp::post()
        .and(warp::path("revoke-guess"))
        .and(warp::path::param::<String>())
        .and(warp::query::<UserIdQueryParam>())
        .and_then({
            let rooms = rooms.clone();
            let clients_sockets = clients_sockets.clone();
            move |room_id: String, UserIdQueryParam { user_id }: UserIdQueryParam| {
                let rooms = rooms.clone();
                let clients_sockets = clients_sockets.clone();
                async move { handlers::revoke_guess(rooms, room_id, user_id, clients_sockets).await }
            }
        })
        .with(cors.clone());

    let acquire_id = warp::path("acquire-id")
        .and_then(move || async move { handlers::acquire_id().await })
        .with(cors.clone());

    let healthcheck = warp::path("healthcheck")
        .and_then(move || async move { handlers::healthcheck().await })
        .with(cors.clone());

    let routes = chat
        .or(can_connect)
        .or(is_host)
        .or(create_room)
        .or(users_of_room)
        .or(messages_of_room)
        .or(submit_guess)
        .or(revoke_guess)
        .or(acquire_id)
        .or(healthcheck)
        .with(cors);

    warp::serve(routes).run(([0, 0, 0, 0], 3030)).await;
}
