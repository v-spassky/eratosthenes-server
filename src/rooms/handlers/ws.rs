use crate::app_context::{AppContext, RequestContext};
use crate::rooms::bot_messages::BotMessage;
use crate::rooms::consts::MAX_MESSAGE_LENGTH;
use crate::rooms::consts::ROUNDS_PER_GAME;
use crate::rooms::message_types::{
    self, BriefUserInfoPayload, ClientSentSocketMessage, ServerSentChatMessagePayload,
    ServerSentSocketMessage,
};
use crate::rooms::models::ChatMessage;
use crate::storage::interface::IRoomStorage;
use crate::storage::rooms::UserConnectedResult;
use futures_util::{
    stream::{SplitSink, SplitStream},
    SinkExt, StreamExt, TryFutureExt,
};
use tokio::sync::mpsc;
use tokio_stream::wrappers::UnboundedReceiverStream;
use unicode_segmentation::UnicodeSegmentation;
use warp::ws::{Message, WebSocket};

pub struct RoomWsHandler<RS: IRoomStorage> {
    app_context: AppContext<RS>,
    request_context: RequestContext,
    socket_id: usize,
    user_ws_tx: Option<SplitSink<WebSocket, Message>>,
    user_ws_rx: SplitStream<WebSocket>,
    rx: Option<UnboundedReceiverStream<Message>>,
}

impl<RS> RoomWsHandler<RS>
where
    RS: IRoomStorage,
{
    pub async fn new(
        app_context: AppContext<RS>,
        request_context: RequestContext,
        websocket: WebSocket,
    ) -> Self {
        // Split the socket into a sender and receiver of messages.
        // Use an unbounded channel to handle buffering and flushing of messages to the websocket.
        let (user_ws_tx, user_ws_rx) = websocket.split();
        let (tx, rx) = mpsc::unbounded_channel();
        let rx = UnboundedReceiverStream::new(rx);
        let socket_id = app_context.sockets.add(tx).await;
        Self {
            app_context,
            request_context,
            socket_id,
            user_ws_tx: Some(user_ws_tx),
            user_ws_rx,
            rx: Some(rx),
        }
    }

    pub async fn on_user_connected(&mut self) {
        let mut user_ws_tx = self.user_ws_tx.take().unwrap();
        let mut rx = self.rx.take().unwrap();
        tokio::task::spawn(async move {
            while let Some(message) = rx.next().await {
                user_ws_tx
                    .send(message)
                    .unwrap_or_else(|e| eprintln!("[user_connected]: websocket send error: {e}"))
                    .await;
            }
        });
        let socket_id = self.socket_id;
        while let Some(result) = self.user_ws_rx.next().await {
            let message = match result {
                Ok(message) => message,
                Err(e) => {
                    eprintln!("[user_connected]: websocket error(uid={socket_id}): {e}");
                    break;
                }
            };
            self.on_new_message(message).await;
        }
        self.on_user_disconnected().await;
    }

    async fn on_new_message(&self, msg: Message) {
        let raw_incoming_msg = if let Ok(s) = msg.to_str() {
            s
        } else {
            eprintln!("[user_message]: error deserializing such message (1): {msg:?}");
            return;
        };

        let socket_message: Result<ClientSentSocketMessage, _> =
            serde_json::from_str(raw_incoming_msg);
        if socket_message.is_err() {
            eprintln!(
                "[user_message]: error deserializing such message (2): {raw_incoming_msg:?}, \
                {socket_message:?}"
            );
            return;
        }
        let relevant_socket_ids = self
            .app_context
            .rooms
            .socket_ids_except_sender(&self.request_context.room_id, self.socket_id)
            .await;
        let socket_message = socket_message.unwrap();
        match socket_message {
            ClientSentSocketMessage::ChatMessage { payload, .. } => {
                if self
                    .app_context
                    .rooms
                    .is_muted(
                        &self.request_context.room_id,
                        &self.request_context.public_id,
                    )
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
                let chat_message =
                    ChatMessage::new(false, Some(payload.from.clone()), payload.content.clone());
                let ws_chat_message = ServerSentSocketMessage::ChatMessage {
                    r#type: message_types::ChatMessage,
                    payload: ServerSentChatMessagePayload {
                        id: chat_message.id,
                        from: Some(payload.from),
                        content: payload.content,
                        is_from_bot: false,
                    },
                };
                let raw_chat_message = serde_json::to_string(&ws_chat_message).unwrap();
                self.app_context
                    .rooms
                    .add_message(&self.request_context.room_id, chat_message)
                    .await;
                self.app_context
                    .sockets
                    .broadcast_msg(&raw_chat_message, &relevant_socket_ids)
                    .await;
            }
            ClientSentSocketMessage::UserConnected { payload, .. } => {
                if !self
                    .app_context
                    .rooms
                    .has_user_with_such_private_id(
                        &self.request_context.room_id,
                        &self.request_context.private_id,
                    )
                    .await
                {
                    let bot_chat_msg = BotMessage::UserConnected {
                        username: &payload.username,
                    };
                    let raw_bot_chat_msg = bot_chat_msg.to_human_readable();
                    let bot_message = ChatMessage::new(true, None, raw_bot_chat_msg.clone());
                    let ws_message = ServerSentSocketMessage::ChatMessage {
                        r#type: message_types::ChatMessage,
                        payload: ServerSentChatMessagePayload {
                            id: bot_message.id,
                            from: None,
                            content: raw_bot_chat_msg,
                            is_from_bot: true,
                        },
                    };
                    let msg = serde_json::to_string(&ws_message).unwrap();
                    let mut all_sockets_ids = relevant_socket_ids.clone();
                    all_sockets_ids.push(Some(self.socket_id));
                    let ws_event = ServerSentSocketMessage::UserConnected {
                        r#type: message_types::UserConnected,
                        payload: BriefUserInfoPayload {
                            username: payload.username.clone(),
                            avatar_emoji: payload.avatar_emoji.clone(),
                        },
                    };
                    let raw_ws_event = serde_json::to_string(&ws_event).unwrap();
                    self.app_context
                        .rooms
                        .add_message(&self.request_context.room_id, bot_message)
                        .await;
                    self.app_context
                        .sockets
                        .broadcast_msg(&msg, &all_sockets_ids)
                        .await;
                    self.app_context
                        .sockets
                        .broadcast_msg(&raw_ws_event, &relevant_socket_ids)
                        .await;
                }
                match self
                    .app_context
                    .rooms
                    .on_user_connected(
                        &self.request_context.room_id,
                        payload,
                        self.socket_id,
                        &self.request_context.public_id,
                        &self.request_context.private_id,
                    )
                    .await
                {
                    Ok(UserConnectedResult::NewUser) => {
                        // TODO
                        self.app_context
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
                self.app_context
                    .rooms
                    .on_user_reconnected(
                        &self.request_context.room_id,
                        payload,
                        self.socket_id,
                        &self.request_context.private_id,
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
                self.app_context
                    .rooms
                    .on_user_disconnected(
                        &self.request_context.room_id,
                        raw_ws_event,
                        &self.request_context.private_id,
                        self.socket_id,
                        self.app_context.sockets.clone(),
                    )
                    .await;
            }
            ClientSentSocketMessage::RoundStarted { .. } => {
                // TODO: Check if the user is host
                let rounds_left = self
                    .app_context
                    .rooms
                    .current_round_number(&self.request_context.room_id)
                    .await;
                let round_number = match rounds_left {
                    0 => ROUNDS_PER_GAME,
                    _ => ROUNDS_PER_GAME + 1 - rounds_left,
                };
                let bot_chat_msg = BotMessage::RoundStarted {
                    round_number,
                    rounds_per_game: ROUNDS_PER_GAME,
                };
                let raw_bot_chat_msg = bot_chat_msg.to_human_readable();
                let bot_message = ChatMessage::new(true, None, raw_bot_chat_msg.clone());
                let ws_message = ServerSentSocketMessage::ChatMessage {
                    r#type: message_types::ChatMessage,
                    payload: ServerSentChatMessagePayload {
                        id: bot_message.id,
                        from: None,
                        content: raw_bot_chat_msg,
                        is_from_bot: true,
                    },
                };
                let msg = serde_json::to_string(&ws_message).unwrap();
                let mut all_sockets_ids = relevant_socket_ids.clone();
                all_sockets_ids.push(Some(self.socket_id));
                self.app_context
                    .rooms
                    .add_message(&self.request_context.room_id, bot_message)
                    .await;
                self.app_context
                    .sockets
                    .broadcast_msg(&msg, &all_sockets_ids)
                    .await;
                self.app_context
                    .rooms
                    .start_game(
                        &self.request_context.room_id,
                        self.app_context.sockets.clone(),
                    )
                    .await;
                let ws_event = ServerSentSocketMessage::RoundStarted {
                    r#type: message_types::RoundStarted,
                };
                let raw_ws_event = serde_json::to_string(&ws_event).unwrap();
                self.app_context
                    .sockets
                    .broadcast_msg(&raw_ws_event, &relevant_socket_ids)
                    .await;
            }
            ClientSentSocketMessage::Ping { .. } => {
                let ws_message = ServerSentSocketMessage::Pong {
                    r#type: message_types::Pong,
                };
                let msg = serde_json::to_string(&ws_message).unwrap();
                self.app_context
                    .sockets
                    .send_msg(&msg, self.socket_id)
                    .await;
            }
        }
    }

    async fn on_user_disconnected(&self) {
        self.app_context.sockets.remove(self.socket_id).await;
        self.app_context
            .rooms
            .disconnect_user(&self.request_context.room_id, self.socket_id)
            .await;
    }
}
