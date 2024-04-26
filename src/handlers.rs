use crate::message_types::{SocketMessage, SocketMessagePayload, SocketMessageType};
use crate::models::LatLng;
use crate::{
    storage::{self, UserConnectedResult},
    user_id,
};
use futures_util::{SinkExt, StreamExt, TryFutureExt};
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
    user_id: String,
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
        user_message(
            socket_id,
            msg,
            client_sockets.clone(),
            &rooms,
            &room_id,
            &user_id,
        )
        .await;
    }
    user_disconnected(socket_id, &client_sockets, &rooms, &room_id, &user_id).await;
}

pub async fn check_if_user_can_connect(
    rooms: storage::Rooms,
    room_id: String,
    user_id: String,
    username: String,
) -> Result<String, Infallible> {
    if !rooms.such_room_exists(&room_id).await {
        return Ok::<_, Infallible>(
            "{\"canConnect\": false, \"reason\": \"Room not found.\"}".to_string(),
        );
    }
    if rooms
        .room_has_user_with_such_username(&room_id, &username, &user_id)
        .await
    {
        return Ok::<_, Infallible>(
            "{\"canConnect\": false, \"reason\": \"Such user already in the room.\"}".to_string(),
        );
    }
    Ok::<_, Infallible>("{\"canConnect\": true}".to_string())
}

pub async fn check_if_user_is_host(
    rooms: storage::Rooms,
    room_id: String,
    user_id: String,
) -> Result<String, Infallible> {
    if !rooms.such_room_exists(&room_id).await {
        return Ok::<_, Infallible>("{\"isHost\": false}".to_string());
    }
    Ok::<_, Infallible>(format!(
        "{{\"isHost\": {}}}",
        rooms.user_is_host_of_the_room(&room_id, &user_id).await
    ))
}

pub async fn get_users_of_room(
    rooms: storage::Rooms,
    room_id: String,
    _user_id: String,
) -> Result<String, Infallible> {
    if !rooms.such_room_exists(&room_id).await {
        return Ok::<_, Infallible>(
            "{\"error\": true, \"reason\": \"Room not found.\"}".to_string(),
        );
    }
    Ok::<_, Infallible>(format!(
        "{{\"error\": false, \"users\": {}, \"status\": {}}}",
        rooms.users_of_room_as_json(&room_id).await,
        rooms.room_status_as_json(&room_id).await,
    ))
}

pub async fn submit_guess(
    rooms: storage::Rooms,
    room_id: String,
    user_id: String,
    guess_json: HashMap<String, String>,
) -> Result<String, Infallible> {
    if !rooms.such_room_exists(&room_id).await {
        return Ok::<_, Infallible>(
            "{\"error\": true, \"reason\": \"Room not found.\"}".to_string(),
        );
    }
    let guess = LatLng {
        lat: guess_json.get("lat").unwrap().parse().unwrap(),
        lng: guess_json.get("lng").unwrap().parse().unwrap(),
    };
    rooms.submit_user_guess(&room_id, &user_id, guess).await;
    Ok::<_, Infallible>(String::from("{\"error\": false}"))
}

pub async fn acquire_id() -> Result<String, Infallible> {
    Ok::<_, Infallible>(format!(
        "{{\"error\": false, \"userId\": \"{}\"}}",
        user_id::generate_user_id(),
    ))
}

pub async fn create_room(rooms: storage::Rooms, _user_id: String) -> Result<String, Infallible> {
    let room_id = rooms.create_room().await;
    Ok::<_, Infallible>(format!("{{\"roomId\": \"{}\"}}", room_id))
}

async fn user_message(
    socket_id: usize,
    msg: Message,
    client_sockets: storage::ClientSockets,
    rooms: &storage::Rooms,
    room_id: &str,
    user_id: &str,
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
            match rooms
                .handle_new_user_connected(room_id, payload, socket_id, user_id)
                .await
            {
                Ok(UserConnectedResult::NewUser) => {}
                Ok(UserConnectedResult::AlreadyInTheRoom) => {
                    return;
                }
                Err(_) => {
                    eprintln!("[user_message]: user with such name already connected : {msg:?}.");
                    return;
                }
            }
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
                .handle_user_reconnected(room_id, payload, socket_id, user_id)
                .await;
            return;
        }
        SocketMessageType::UserDisconnected => {
            let _payload = match socket_message.payload {
                Some(SocketMessagePayload::BriefUserInfo(payload)) => payload,
                _ => {
                    println!("[user_message]: error deserializing such message (5): {msg:?}");
                    return;
                }
            };
            rooms
                .handle_user_disconnected(
                    room_id,
                    msg.to_string(),
                    user_id,
                    socket_id,
                    client_sockets.clone(),
                )
                .await;
            return;
        }
        SocketMessageType::GameStarted => {
            // TODO: Check if the user is host
            rooms
                .handle_game_started(room_id, client_sockets.clone())
                .await;
        }
        SocketMessageType::GameFinished => {
            // TODO: Check if the user is host + this should be coming from the server, not the client
            // rooms.handle_game_finished(room_id).await;
            // TODO: delete this message type and handler
            return;
        }
        SocketMessageType::Ping => {
            if let Err(_disconnected) = client_sockets
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

    let relevant_socket_ids = rooms.relevant_socket_ids(room_id, socket_id).await;

    println!("[user_message]: broadcasting message {msg} to users: {relevant_socket_ids:?}");

    for (&uid, tx) in client_sockets.read().await.iter() {
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
    _user_id: &str,
) {
    eprintln!("[user_disconnected]: good bye user: {user_socket_id}");
    client_sockets.write().await.remove(&user_socket_id);
    rooms.disconnect_user(room_id, user_socket_id).await;
    // client_sockets.write().await.remove(&user_socket_id);
}
