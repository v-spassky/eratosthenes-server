use serde::{Deserialize, Serialize};
use warp::{hyper::Method, Filter};

mod handlers;
mod message_types;
mod models;
mod storage;
mod user_descriptions;

#[derive(Serialize, Deserialize)]
struct CanConnectQueryParams {
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
        .map({
            let clients_sockets = clients_sockets.clone();
            let rooms = rooms.clone();
            move |ws: warp::ws::Ws, room_id| {
                let clients_sockets = clients_sockets.clone();
                let rooms = rooms.clone();
                ws.on_upgrade(|socket| {
                    handlers::user_connected(socket, clients_sockets, room_id, rooms)
                })
            }
        })
        .with(cors.clone());

    let can_connect = warp::path("can-connect")
        .and(warp::path::param::<String>())
        .and(warp::query::<CanConnectQueryParams>())
        .and_then({
            let rooms = rooms.clone();
            move |room_id: String, CanConnectQueryParams { username }: CanConnectQueryParams| {
                let rooms = rooms.clone();
                async move { handlers::check_if_user_can_connect(rooms, room_id, username).await }
            }
        })
        .with(cors.clone());

    let create_room = warp::post()
        .and(warp::path("create-room"))
        .and_then({
            let rooms = rooms.clone();
            move || {
                let rooms = rooms.clone();
                async move { handlers::create_room(rooms).await }
            }
        })
        .with(cors.clone());

    let users_of_room = warp::path("users-of-room")
        .and(warp::path::param::<String>())
        .and_then({
            let rooms = rooms.clone();
            move |room_id: String| {
                let rooms = rooms.clone();
                async move { handlers::get_users_of_room(rooms, room_id).await }
            }
        })
        .with(cors.clone());

    let routes = chat
        .or(can_connect)
        .or(create_room)
        .or(users_of_room)
        .with(cors);

    warp::serve(routes).run(([0, 0, 0, 0], 3030)).await;
}
