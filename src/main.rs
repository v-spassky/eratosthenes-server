use rand::{distributions::Alphanumeric, Rng};
use std::collections::HashMap;
use std::convert::Infallible;
use std::sync::{
    atomic::{AtomicUsize, Ordering},
    Arc,
};
use warp::{hyper::Method, Filter};

use futures_util::{SinkExt, StreamExt, TryFutureExt};
use serde::{Deserialize, Serialize};
use tokio::sync::{mpsc, RwLock};
use tokio_stream::wrappers::UnboundedReceiverStream;
use warp::ws::{Message, WebSocket};

/// Our global unique user id counter.
static NEXT_USER_ID: AtomicUsize = AtomicUsize::new(1);

/// Our state of currently connected users.
///
/// - Key is their id
/// - Value is a sender of `warp::ws::Message`
type ClientSockets = Arc<RwLock<HashMap<usize, mpsc::UnboundedSender<Message>>>>;

type Rooms = Arc<RwLock<HashMap<String, Room>>>;

#[derive(Debug)]
struct Room {
    users: Vec<User>,
    messages: Vec<ChatMessage>,
}

#[derive(Debug)]
struct User {
    name: String,
    avatar_emoji: String,
    socket_id: usize,
}

#[derive(Debug)]
struct ChatMessage {
    author_name: String,
    content: String,
}

#[derive(Serialize, Deserialize)]
struct CanConnectQueryParams {
    username: String,
}

#[tokio::main]
async fn main() {
    let clients_sockets = ClientSockets::default();
    let rooms = Rooms::default();

    let cors = warp::cors()
        .allow_origin("http://127.0.0.1:3000")
        .allow_origin("http://localhost:3000")
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
                ws.on_upgrade(|socket| user_connected(socket, clients_sockets, room_id, rooms))
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
                async move {
                    let room_exists = rooms.read().await.contains_key(&room_id);
                    if !room_exists {
                        return Ok::<_, Infallible>(
                            "{\"canConnect\": false, \"reason\": \"Room not found.\"}".to_string(),
                        );
                    }
                    let room_has_user_with_such_name = rooms
                        .read()
                        .await
                        .get(&room_id)
                        .unwrap()
                        .users
                        .iter()
                        .any(|user| user.name == username);
                    if room_has_user_with_such_name {
                        return Ok::<_, Infallible>(format!(
                            "{{\"canConnect\": false, \"reason\": \"User with name {}
                            already exists in the room.\"}}",
                            username,
                        ));
                    }
                    Ok::<_, Infallible>(format!("{{\"canConnect\": {}}}", room_exists))
                }
            }
        })
        .with(cors.clone());

    let create_room = warp::post()
        .and(warp::path("create-room"))
        .and_then({
            let rooms = rooms.clone();
            move || {
                let rooms = rooms.clone();
                async move {
                    let room_id = generate_room_id();
                    let room = Room {
                        users: vec![],
                        messages: vec![],
                    };
                    rooms.write().await.insert(room_id.clone(), room);
                    Ok::<_, Infallible>(format!("{{\"roomId\": \"{}\"}}", room_id))
                }
            }
        })
        .with(cors.clone());

    let routes = chat.or(can_connect).or(create_room).with(cors);

    warp::serve(routes).run(([127, 0, 0, 1], 3030)).await;
}

async fn user_connected(
    ws: WebSocket,
    client_sockets: ClientSockets,
    room_id: String,
    rooms: Rooms,
) {
    let socket_id = NEXT_USER_ID.fetch_add(1, Ordering::Relaxed);
    rooms
        .write()
        .await
        .get_mut(&room_id)
        .unwrap() // TODO: this `.unwrap()` isn't safe
        .users
        .push(User {
            name: "John Doe".to_string(),
            avatar_emoji: "👤".to_string(),
            socket_id,
        });

    eprintln!("new chat user: {}", socket_id);

    // Split the socket into a sender and receiver of messages.
    // Use an unbounded channel to handle buffering and flushing of messages to the websocket.
    let (mut user_ws_tx, mut user_ws_rx) = ws.split();
    let (tx, rx) = mpsc::unbounded_channel();
    let mut rx = UnboundedReceiverStream::new(rx);

    tokio::task::spawn(async move {
        while let Some(message) = rx.next().await {
            user_ws_tx
                .send(message)
                .unwrap_or_else(|e| eprintln!("websocket send error: {}", e))
                .await;
        }
    });

    client_sockets.write().await.insert(socket_id, tx);

    while let Some(result) = user_ws_rx.next().await {
        let msg = match result {
            Ok(msg) => msg,
            Err(e) => {
                eprintln!("websocket error(uid={}): {}", socket_id, e);
                break;
            }
        };
        user_message(socket_id, msg, &client_sockets, &rooms, &room_id).await;
    }
    user_disconnected(socket_id, &client_sockets).await;
}

async fn user_message(
    socket_id: usize,
    msg: Message,
    users: &ClientSockets,
    rooms: &Rooms,
    room_id: &str,
) {
    let msg = if let Ok(s) = msg.to_str() {
        s
    } else {
        return;
    };

    let relevant_socket_ids = rooms
        .read()
        .await
        .get(room_id)
        .unwrap() // TODO: this `.unwrap()` isn't safe
        .users
        .iter()
        .filter(|user| user.socket_id != socket_id)
        .map(|user| user.socket_id)
        .collect::<Vec<_>>();

    // New message from this user, send it to everyone else (except same uid)...
    for (&uid, tx) in users.read().await.iter() {
        if relevant_socket_ids.contains(&uid) {
            if let Err(_disconnected) = tx.send(Message::text(msg.to_string())) {
                // The tx is disconnected, our `user_disconnected` code
                // should be happening in another task, nothing more to
                // do here.
            }
        }
    }
}

async fn user_disconnected(my_id: usize, client_sockets: &ClientSockets) {
    eprintln!("good bye user: {}", my_id);
    client_sockets.write().await.remove(&my_id);
}

fn generate_room_id() -> String {
    rand::thread_rng()
        .sample_iter(&Alphanumeric)
        .take(10)
        .map(char::from)
        .collect()
}
