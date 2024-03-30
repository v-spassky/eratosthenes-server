use crate::message_types::{SocketMessage, SocketMessagePayload, SocketMessageType};
use crate::models;
use crate::storage;
use futures_util::{SinkExt, StreamExt, TryFutureExt};
use rand::{distributions::Alphanumeric, Rng};
use std::collections::HashMap;
use std::convert::Infallible;
use std::sync::atomic::Ordering;
use tokio::sync::mpsc;
use tokio_stream::wrappers::UnboundedReceiverStream;
use warp::ws::{Message, WebSocket};

pub async fn user_connected(
    ws: WebSocket,
    client_sockets: storage::ClientSockets,
    room_id: String,
    rooms: storage::Rooms,
) {
    let socket_id = storage::NEXT_USER_ID.fetch_add(1, Ordering::Relaxed);

    println!("[user_connected]: assigned ID {socket_id} to the new user.");

    // Split the socket into a sender and receiver of messages.
    // Use an unbounded channel to handle buffering and flushing of messages to the websocket.
    let (mut user_ws_tx, mut user_ws_rx) = ws.split();
    let (tx, rx) = mpsc::unbounded_channel();
    let mut rx = UnboundedReceiverStream::new(rx);

    tokio::task::spawn(async move {
        while let Some(message) = rx.next().await {
            user_ws_tx
                .send(message)
                .unwrap_or_else(|e| eprintln!("[user_connected]: websocket send error: {e}"))
                .await;
        }
    });

    client_sockets.write().await.insert(socket_id, tx);

    while let Some(result) = user_ws_rx.next().await {
        let msg = match result {
            Ok(msg) => msg,
            Err(e) => {
                eprintln!("[user_connected]: websocket error(uid={socket_id}): {e}");
                break;
            }
        };
        user_message(socket_id, msg, &client_sockets, &rooms, &room_id).await;
    }
    user_disconnected(socket_id, &client_sockets, &rooms, &room_id).await;
}

pub async fn check_if_user_can_connect(
    rooms: storage::Rooms,
    room_id: String,
    username: String,
) -> Result<String, Infallible> {
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

pub async fn check_if_user_is_host(
    rooms: storage::Rooms,
    room_id: String,
    username: String,
) -> Result<String, Infallible> {
    let room_exists = rooms.read().await.contains_key(&room_id);
    if !room_exists {
        return Ok::<_, Infallible>("{\"isHost\": false}".to_string());
    }
    let user_is_host = rooms
        .read()
        .await
        .get(&room_id)
        .unwrap()
        .users
        .iter()
        .find(|user| user.name == username)
        .map_or(false, |user| user.is_host);
    Ok::<_, Infallible>(format!("{{\"isHost\": {}}}", user_is_host))
}

pub async fn get_users_of_room(
    rooms: storage::Rooms,
    room_id: String,
) -> Result<String, Infallible> {
    let room_exists = rooms.read().await.contains_key(&room_id);
    if !room_exists {
        return Ok::<_, Infallible>(
            "{\"error\": true, \"reason\": \"Room not found.\"}".to_string(),
        );
    }
    // TODO: this `.unwrap()` isn't safe
    let users = rooms.read().await.get(&room_id).unwrap().users_as_json();
    let room_status = rooms.read().await.get(&room_id).unwrap().status.as_json();
    Ok::<_, Infallible>(format!(
        "{{\"error\": false, \"users\": {}, \"status\": {}}}",
        users, room_status
    ))
}

pub async fn submit_guess(
    rooms: storage::Rooms,
    room_id: String,
    guess_json: HashMap<String, String>,
) -> Result<String, Infallible> {
    let room_exists = rooms.read().await.contains_key(&room_id);
    if !room_exists {
        return Ok::<_, Infallible>(
            "{\"error\": true, \"reason\": \"Room not found.\"}".to_string(),
        );
    }
    let username = guess_json.get("username").unwrap();
    let guess = models::LatLng {
        lat: guess_json.get("lat").unwrap().parse().unwrap(),
        lng: guess_json.get("lng").unwrap().parse().unwrap(),
    };
    // TODO: this `.unwrap()` isn't safe
    rooms
        .write()
        .await
        .get_mut(&room_id)
        .unwrap()
        .users
        .iter_mut()
        .find(|user| user.name == *username)
        .unwrap()
        .submit_guess(guess);
    Ok::<_, Infallible>(String::from("{\"error\": false}"))
}

pub async fn create_room(rooms: storage::Rooms) -> Result<String, Infallible> {
    let room_id = generate_room_id();
    let room = models::Room {
        users: vec![],
        messages: vec![],
        status: models::RoomStatus::Waiting {
            previous_location: None,
        },
    };
    rooms.write().await.insert(room_id.clone(), room);
    Ok::<_, Infallible>(format!("{{\"roomId\": \"{}\"}}", room_id))
}

async fn user_message(
    socket_id: usize,
    msg: Message,
    users: &storage::ClientSockets,
    rooms: &storage::Rooms,
    room_id: &str,
) {
    let msg = if let Ok(s) = msg.to_str() {
        s
    } else {
        eprintln!("[user_message]: error deserializing such message (1): {msg:?}");
        return;
    };

    let socket_message: Result<SocketMessage, _> = serde_json::from_str(msg);
    if socket_message.is_err() {
        eprintln!(
            "[user_message]: error deserializing such message (2): {msg:?}, {socket_message:?}"
        );
        return;
    }
    let socket_message = socket_message.unwrap();
    match socket_message.r#type {
        SocketMessageType::ChatMessage => {}
        SocketMessageType::UserConnected => {
            let payload = match socket_message.payload {
                Some(SocketMessagePayload::BriefUserInfo(payload)) => payload,
                _ => {
                    eprintln!(
                        "[user_message]: error deserializing such message (3): {:?}",
                        msg
                    );
                    return;
                }
            };
            let room_has_no_members = rooms
                .read()
                .await
                .get(room_id)
                .unwrap() // TODO: this `.unwrap()` isn't safe
                .users
                .is_empty();
            let description_ids_of_room_members = rooms
                .read()
                .await
                .get(room_id)
                .unwrap() // TODO: this `.unwrap()` isn't safe
                .users
                .iter()
                .map(|user| user.description_id)
                .collect::<Vec<_>>();
            let such_user_already_connected = rooms
                .read()
                .await
                .get(room_id)
                .unwrap() // TODO: this `.unwrap()` isn't safe
                .users
                .iter()
                .any(|user| user.name == payload.username);
            if such_user_already_connected {
                eprintln!("[user_message]: user with such name already connected : {payload:?}.");
                return;
            }
            println!("[user_message]: adding user {payload:?} to room {room_id}.");
            rooms
                .write()
                .await
                .get_mut(room_id)
                .unwrap() // TODO: this `.unwrap()` isn't safe
                .users
                .push(models::User::new(
                    payload.username,
                    payload.avatar_emoji,
                    room_has_no_members,
                    description_ids_of_room_members,
                    socket_id,
                ));
        }
        SocketMessageType::UserReConnected => {
            let payload = match socket_message.payload {
                Some(SocketMessagePayload::BriefUserInfo(payload)) => payload,
                _ => {
                    println!("[user_message]: error deserializing such message (4): {msg:?}");
                    return;
                }
            };
            rooms
                .write()
                .await
                .get_mut(room_id)
                .unwrap() // TODO: this `.unwrap()` isn't safe
                .users
                .iter_mut()
                .find(|user| user.name == payload.username)
                .unwrap()
                .socket_id = Some(socket_id);
            return;
        }
        SocketMessageType::UserDisconnected => {
            let payload = match socket_message.payload {
                Some(SocketMessagePayload::BriefUserInfo(payload)) => payload,
                _ => {
                    println!("[user_message]: error deserializing such message (5): {msg:?}");
                    return;
                }
            };
            let user_is_host = rooms
                .read()
                .await
                .get(room_id)
                .unwrap() // TODO: this `.unwrap()` isn't safe
                .users
                .iter()
                .find(|user| user.name == payload.username)
                .unwrap() // TODO: this `.unwrap()` isn't safe
                .is_host;
            rooms
                .write()
                .await
                .get_mut(room_id)
                .unwrap() // TODO: this `.unwrap()` isn't safe
                .users
                .retain(|user| user.name != payload.username);
            if user_is_host {
                rooms
                    .write()
                    .await
                    .get_mut(room_id)
                    .unwrap() // TODO: this `.unwrap()` isn't safe
                    .reassign_host()
            }
        }
        SocketMessageType::GameStarted => {
            rooms
                .write()
                .await
                .get_mut(room_id)
                .unwrap()
                .start_playing();
        }
        SocketMessageType::GameFinished => {
            rooms
                .write()
                .await
                .get_mut(room_id)
                .unwrap() // TODO: this `.unwrap()` isn't safe
                .finish_game();
        }
        SocketMessageType::Ping => {
            if let Err(_disconnected) = users
                .read()
                .await
                .get(&socket_id)
                .unwrap()
                .send(Message::text("{\"type\": \"pong\", \"payload\": null}"))
            {
                // The tx is disconnected, our `user_disconnected` code
                // should be happening in another task, nothing more to
                // do here.
                eprintln!("[user_message]: error sending pong to user: {socket_id:?}")
            }
            return;
        }
    }

    let relevant_socket_ids = rooms
        .read()
        .await
        .get(room_id)
        .unwrap() // TODO: this `.unwrap()` isn't safe
        .users
        .iter()
        .filter(|user| user.socket_id != Some(socket_id)) // TODO: maybe compare by username ?
        .map(|user| user.socket_id)
        .collect::<Vec<_>>();

    println!("[user_message]: broadcasting message {msg} to users: {relevant_socket_ids:?}");

    for (&uid, tx) in users.read().await.iter() {
        if relevant_socket_ids.contains(&Some(uid)) {
            if let Err(_disconnected) = tx.send(Message::text(msg.to_string())) {
                // The tx is disconnected, our `user_disconnected` code
                // should be happening in another task, nothing more to
                // do here.
                eprintln!(
                    "[user_message]: error broadcasting message {msg} to user ith id {uid:?}"
                );
            }
        }
    }
}

async fn user_disconnected(
    user_socket_id: usize,
    client_sockets: &storage::ClientSockets,
    rooms: &storage::Rooms,
    room_id: &str,
) {
    eprintln!("[user_disconnected]: good bye user: {user_socket_id}");
    client_sockets.write().await.remove(&user_socket_id);
    let mut room_lock = rooms.write().await;
    let user = room_lock
        .get_mut(room_id)
        .unwrap()
        .users
        .iter_mut()
        .find(|user| user.socket_id == Some(user_socket_id));
    match user {
        Some(user) => {
            // socket closed not on behalf of the user
            user.socket_id = None;
        }
        None => {
            println!("[user_disconnected]: user with such socket id not found: {user_socket_id}");
        }
    }
    client_sockets.write().await.remove(&user_socket_id);
}

fn generate_room_id() -> String {
    rand::thread_rng()
        .sample_iter(&Alphanumeric)
        .take(10)
        .map(char::from)
        .collect()
}
