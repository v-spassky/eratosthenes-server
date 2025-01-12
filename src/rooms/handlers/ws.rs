use crate::app_context::{AppContext, RequestContext};
use crate::auth::passcode;
use crate::http::requests::PasscodeQueryParam;
use crate::rooms::consts::MAX_MESSAGE_LENGTH;
use crate::rooms::consts::ROUNDS_PER_GAME;
use crate::rooms::message_types::{
    self, BotMessagePayload, BriefUserInfoPayload, ClientSentSocketMessage,
    RoundStartedBotMessagePayload, RoundStartedBotMsg, ServerSentChatMessagePayload,
    ServerSentSocketMessage, UserConnectedBotMessagePayload, UserConnectedBotMsg,
};
use crate::rooms::models::ChatMessage;
use crate::storage::interface::{
    RoomConnectionHandler, RoomGameFlowHandler, RoomRepo, RoomSocketsRepo, UserPermissionsRepo,
};
use crate::storage::rooms::HashMapRoomsStorage;
use crate::storage::rooms::UserConnectedResult;
use axum::extract::ws::Message;
use axum::extract::ws::WebSocket;
use axum::extract::{Path, Query, State, WebSocketUpgrade};
use axum::response::Response;
use futures_util::{SinkExt, StreamExt, TryFutureExt};
use std::time::{SystemTime, UNIX_EPOCH};
use tokio::sync::mpsc;
use tokio::time::Instant;
use tokio_stream::wrappers::UnboundedReceiverStream;
use unicode_segmentation::UnicodeSegmentation;

// TODO: make the handler generic over rooms storage

pub async fn ws(
    ws: WebSocketUpgrade,
    Path(room_id): Path<String>,
    Query(query_params): Query<PasscodeQueryParam>,
    State(app_context): State<AppContext<HashMapRoomsStorage>>,
) -> Response {
    // TODO: If the handler is made generic over rooms storage, `ws.on_upgrade` requires the future
    // returned by the closure to be `Send`, so all futures that it awaits must also be `Send`.
    // Compiler diagnostics:
    //
    // help: `std::marker::Send` can be made part of the associated future's guarantees for all
    // implementations of `RoomConnectionHandler::disconnect_user`
    // --> src/storage/interface.rs:77:5
    //
    // - async fn disconnect_user(...);
    // + fn disconnect_user(...) -> impl std::future::Future<Output = ()> + std::marker::Send;
    ws.on_upgrade(|socket| handle_socket(socket, room_id, query_params.passcode, app_context))
}

async fn handle_socket(
    socket: WebSocket,
    room_id: String,
    passcode: String,
    app_context: AppContext<HashMapRoomsStorage>,
) {
    // TODO: reject if incorrect
    let jwt_payload = passcode::decode(&passcode).unwrap();
    let request_context = RequestContext {
        public_id: jwt_payload.public_id,
        private_id: jwt_payload.private_id,
        room_id,
        // client_ip,
    };
    let (mut user_ws_tx, mut user_ws_rx) = socket.split();
    let (tx, rx) = mpsc::unbounded_channel();
    let mut rx = UnboundedReceiverStream::new(rx);
    let socket_id = app_context.sockets.add(tx).await;

    tokio::task::spawn(async move {
        while let Some(message) = rx.next().await {
            user_ws_tx
                .send(message)
                .unwrap_or_else(|e| eprintln!("[user_connected]: websocket send error: {e}"))
                .await;
        }
    });

    while let Some(result) = user_ws_rx.next().await {
        let message = match result {
            Ok(message) => message,
            Err(e) => {
                eprintln!("[user_connected]: websocket error(uid={socket_id}): {e}");
                break;
            }
        };
        // TODO: is `clone()` needed?
        on_new_message(
            app_context.clone(),
            request_context.clone(),
            message,
            socket_id,
        )
        .await;
    }
    on_user_disconnected(app_context, request_context, socket_id).await;
}

async fn on_new_message(
    app_context: AppContext<HashMapRoomsStorage>,
    request_context: RequestContext,
    msg: Message,
    socket_id: usize,
) {
    let start_time = Instant::now();
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs();
    let raw_incoming_msg = if let Ok(s) = msg.to_text() {
        s
    } else {
        eprintln!("[user_message]: error deserializing such message (1): {msg:?}");
        return;
    };

    let socket_message: Result<ClientSentSocketMessage, _> = serde_json::from_str(raw_incoming_msg);
    if socket_message.is_err() {
        eprintln!(
            "[user_message]: error deserializing such message (2): {raw_incoming_msg:?}, \
                {socket_message:?}"
        );
        return;
    }
    let relevant_socket_ids = app_context
        .rooms
        .socket_ids_except_sender(&request_context.room_id, socket_id)
        .await;
    let socket_message = socket_message.unwrap();
    let message_type = socket_message.message_type_as_string();
    match socket_message {
        ClientSentSocketMessage::ChatMessage { payload, .. } => {
            if app_context
                .rooms
                .is_muted(&request_context.room_id, &request_context.public_id)
                .await
            {
                return;
            }
            if payload.content.graphemes(true).count() > MAX_MESSAGE_LENGTH {
                eprintln!(
                    "Rejecting a message because the it is too long: \
                        {} symbols when at most {} is allowed.",
                    payload.content.len(),
                    MAX_MESSAGE_LENGTH,
                );
                return;
            }
            let chat_message = ChatMessage::from_player(
                payload.from.clone(),
                payload.content.clone(),
                payload.attachment_ids.clone(),
            );
            let ws_chat_message = ServerSentSocketMessage::ChatMessage {
                r#type: message_types::ChatMessage,
                payload: ServerSentChatMessagePayload {
                    id: chat_message.id(),
                    from: payload.from,
                    content: payload.content,
                    attachment_ids: payload.attachment_ids,
                },
            };
            let raw_chat_message = serde_json::to_string(&ws_chat_message).unwrap();
            app_context
                .rooms
                .add_message(&request_context.room_id, chat_message)
                .await;
            app_context
                .sockets
                .broadcast_msg(&raw_chat_message, &relevant_socket_ids)
                .await;
        }
        ClientSentSocketMessage::UserConnected { payload, .. } => {
            if !app_context
                .rooms
                .has_user_with_such_private_id(
                    &request_context.room_id,
                    &request_context.private_id,
                )
                .await
            {
                let bot_message_payload = BotMessagePayload::UserConnected {
                    r#type: UserConnectedBotMsg,
                    payload: UserConnectedBotMessagePayload {
                        username: payload.username.clone(),
                    },
                };
                let bot_message = ChatMessage::from_bot(bot_message_payload.clone());
                let ws_message = ServerSentSocketMessage::BotMessage {
                    r#type: message_types::BotMessage,
                    id: bot_message.id(),
                    payload: bot_message_payload,
                };
                let msg = serde_json::to_string(&ws_message).unwrap();
                let mut all_sockets_ids = relevant_socket_ids.clone();
                all_sockets_ids.push(Some(socket_id));
                let ws_event = ServerSentSocketMessage::UserConnected {
                    r#type: message_types::UserConnected,
                    payload: BriefUserInfoPayload {
                        username: payload.username.clone(),
                        avatar_emoji: payload.avatar_emoji.clone(),
                    },
                };
                let raw_ws_event = serde_json::to_string(&ws_event).unwrap();
                app_context
                    .rooms
                    .add_message(&request_context.room_id, bot_message)
                    .await;
                app_context
                    .sockets
                    .broadcast_msg(&msg, &all_sockets_ids)
                    .await;
                app_context
                    .sockets
                    .broadcast_msg(&raw_ws_event, &relevant_socket_ids)
                    .await;
            }
            match app_context
                .rooms
                .on_user_connected(
                    &request_context.room_id,
                    payload,
                    socket_id,
                    &request_context.public_id,
                    &request_context.private_id,
                )
                .await
            {
                Ok(UserConnectedResult::NewUser) => {
                    // TODO
                    app_context
                        .sockets
                        .broadcast_msg(raw_incoming_msg, &relevant_socket_ids)
                        .await;
                }
                Ok(UserConnectedResult::AlreadyInTheRoom) => {}
                Err(_) => {
                    eprintln!(
                            "[user_message]: user with such name already connected : {raw_incoming_msg:?}."
                        );
                }
            }
        }
        ClientSentSocketMessage::UserReConnected { payload, .. } => {
            app_context
                .rooms
                .on_user_reconnected(
                    &request_context.room_id,
                    payload,
                    socket_id,
                    &request_context.private_id,
                )
                .await;
        }
        ClientSentSocketMessage::UserDisconnected { payload, .. } => {
            let ws_event = ServerSentSocketMessage::UserDisconnected {
                r#type: message_types::UserDisconnected,
                payload: BriefUserInfoPayload {
                    username: payload.username.clone(),
                    avatar_emoji: payload.avatar_emoji.clone(),
                },
            };
            let raw_ws_event = serde_json::to_string(&ws_event).unwrap();
            app_context
                .rooms
                .on_user_disconnected(
                    &request_context.room_id,
                    raw_ws_event,
                    &request_context.private_id,
                    socket_id,
                    app_context.sockets.clone(),
                )
                .await;
        }
        ClientSentSocketMessage::RoundStarted { .. } => {
            // TODO: Check if the user is host
            let rounds_left = app_context
                .rooms
                .current_round_number(&request_context.room_id)
                .await;
            let round_number = match rounds_left {
                0 => ROUNDS_PER_GAME,
                _ => ROUNDS_PER_GAME + 1 - rounds_left,
            };
            let bot_message_payload = BotMessagePayload::RoundStarted {
                r#type: RoundStartedBotMsg,
                payload: RoundStartedBotMessagePayload {
                    round_number,
                    rounds_per_game: ROUNDS_PER_GAME,
                },
            };
            let bot_message = ChatMessage::from_bot(bot_message_payload.clone());
            let ws_message = ServerSentSocketMessage::BotMessage {
                r#type: message_types::BotMessage,
                id: bot_message.id(),
                payload: bot_message_payload,
            };
            let msg = serde_json::to_string(&ws_message).unwrap();
            let mut all_sockets_ids = relevant_socket_ids.clone();
            all_sockets_ids.push(Some(socket_id));
            app_context
                .rooms
                .add_message(&request_context.room_id, bot_message)
                .await;
            app_context
                .sockets
                .broadcast_msg(&msg, &all_sockets_ids)
                .await;
            app_context
                .rooms
                .start_game(&request_context.room_id, app_context.sockets.clone())
                .await;
            let ws_event = ServerSentSocketMessage::RoundStarted {
                r#type: message_types::RoundStarted,
            };
            let raw_ws_event = serde_json::to_string(&ws_event).unwrap();
            app_context
                .sockets
                .broadcast_msg(&raw_ws_event, &relevant_socket_ids)
                .await;
        }
        ClientSentSocketMessage::Ping { .. } => {
            let ws_message = ServerSentSocketMessage::Pong {
                r#type: message_types::Pong,
            };
            let msg = serde_json::to_string(&ws_message).unwrap();
            app_context.sockets.send_msg(&msg, socket_id).await;
        }
    }
    let processing_time_ns = start_time.elapsed().as_nanos();
    tracing::info!(
        task = "client_sent_ws_message",
        message_type = message_type,
        private_id = request_context.private_id,
        // TODO: fix `client_ip`
        client_ip = "127.0.0.1",
        processing_time_ms = processing_time_ns / 1000,
        timestamp,
    );
}

async fn on_user_disconnected(
    app_context: AppContext<HashMapRoomsStorage>,
    request_context: RequestContext,
    socket_id: usize,
) {
    app_context.sockets.remove(socket_id).await;
    app_context
        .rooms
        .disconnect_user(&request_context.room_id, socket_id)
        .await;
}
