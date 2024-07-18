use crate::app_context::{AppContext, RequestContext};
use crate::map_locations::models::LatLng;
use crate::rooms::bot_messages::BotMessage;
use crate::rooms::consts::ROUNDS_PER_GAME;
use crate::rooms::message_types::{
    ChatMessagePayload, SocketMessage, SocketMessagePayload, SocketMessageType, UserPubIdInfoPayload,
};
use crate::rooms::models::ChatMessage;
use crate::storage::interface::IRoomStorage;
use crate::users::responses::{
    BanUserResponse, ChangeScoreResponse, GuessRevocationError, GuessSubmissionError,
    IsUserTheHostResponse, MuteUserResponse, RevokeGuessResponse, ScoreChangeError,
    SubmitGuessResponse, UnmuteUserResponse, UserBanningError, UserMutingError, UserUnmutingError,
};

pub struct UsersHttpHandler<RS: IRoomStorage> {
    app_context: AppContext<RS>,
    request_context: RequestContext,
}

impl<RS> UsersHttpHandler<RS>
where
    RS: IRoomStorage,
{
    pub fn new(app_context: AppContext<RS>, request_context: RequestContext) -> Self {
        Self {
            app_context,
            request_context,
        }
    }

    pub async fn is_host(&self) -> IsUserTheHostResponse {
        if !self.app_context.rooms.exists(&self.request_context.room_id).await {
            return IsUserTheHostResponse { is_host: false };
        }
        let is_host = self
            .app_context
            .rooms
            .user_is_host(&self.request_context.room_id, &self.request_context.public_id)
            .await;
        IsUserTheHostResponse { is_host }
    }

    pub async fn submit_guess(&self, guess: LatLng) -> SubmitGuessResponse {
        if !self.app_context.rooms.exists(&self.request_context.room_id).await {
            return SubmitGuessResponse {
                error: true,
                error_code: Some(GuessSubmissionError::RoomNotFound),
            };
        }
        let round_finished = self
            .app_context
            .rooms
            .submit_guess(&self.request_context.room_id, &self.request_context.private_id, guess)
            .await;
        let room_sockets_ids = self.app_context.rooms.all_socket_ids(&self.request_context.room_id).await;
        let msg = SocketMessage {
            r#type: SocketMessageType::GuessSubmitted,
            payload: None,
        };
        let msg = serde_json::to_string(&msg).unwrap();
        self.app_context
            .sockets
            .broadcast_msg(&msg, &room_sockets_ids)
            .await;
        if round_finished {
            let game_finished = self.app_context.rooms.finish_game(&self.request_context.room_id).await;
            let event_msg = match game_finished {
                true => SocketMessage {
                    r#type: SocketMessageType::GameFinished,
                    payload: None,
                },
                false => SocketMessage {
                    r#type: SocketMessageType::RoundFinished,
                    payload: None,
                },
            };
            let raw_event_msg = serde_json::to_string(&event_msg).unwrap();
            let rounds_left = self
                .app_context
                .rooms
                .current_round_number(&self.request_context.room_id)
                .await;
            let round_number = match rounds_left {
                ROUNDS_PER_GAME => ROUNDS_PER_GAME,
                _ => ROUNDS_PER_GAME - rounds_left,
            };
            let bot_chat_msg = BotMessage::RoundEnded {
                round_number,
                rounds_per_game: ROUNDS_PER_GAME,
            };
            let raw_bot_chat_msg = bot_chat_msg.to_human_readable();
            let bot_ws_msg = SocketMessage {
                r#type: SocketMessageType::ChatMessage,
                payload: Some(SocketMessagePayload::ChatMessage(ChatMessagePayload {
                    from: None,
                    content: raw_bot_chat_msg.clone(),
                    is_from_bot: true,
                })),
            };
            let raw_bot_ws_msg = serde_json::to_string(&bot_ws_msg).unwrap();
            let bot_message = ChatMessage::new(true, None, raw_bot_chat_msg);
            self.app_context
                .rooms
                .add_message(&self.request_context.room_id, bot_message)
                .await;
            self.app_context
                .sockets
                .broadcast_msg(&raw_bot_ws_msg, &room_sockets_ids)
                .await;
            self.app_context
                .sockets
                .broadcast_msg(&raw_event_msg, &room_sockets_ids)
                .await;
        }
        SubmitGuessResponse {
            error: false,
            error_code: None,
        }
    }

    pub async fn revoke_guess(&self) -> RevokeGuessResponse {
        if !self.app_context.rooms.exists(&self.request_context.room_id).await {
            return RevokeGuessResponse {
                error: true,
                error_code: Some(GuessRevocationError::RoomNotFound),
            };
        }
        self.app_context
            .rooms
            .revoke_guess(&self.request_context.room_id, &self.request_context.private_id)
            .await;
        let room_sockets_ids = self.app_context.rooms.all_socket_ids(&self.request_context.room_id).await;
        let msg = SocketMessage {
            r#type: SocketMessageType::GuessRevoked,
            payload: None,
        };
        let msg = serde_json::to_string(&msg).unwrap();
        self.app_context
            .sockets
            .broadcast_msg(&msg, &room_sockets_ids)
            .await;
        RevokeGuessResponse {
            error: false,
            error_code: None,
        }
    }

    pub async fn mute(&self, target_user_public_id: String) -> MuteUserResponse {
        if !self.app_context.rooms.exists(&self.request_context.room_id).await {
            return MuteUserResponse {
                error: true,
                error_code: Some(UserMutingError::RoomNotFound),
            };
        }
        if !self
            .app_context
            .rooms
            .user_is_host(&self.request_context.room_id, &self.request_context.public_id)
            .await
        {
            return MuteUserResponse {
                error: true,
                error_code: Some(UserMutingError::YouAreNotTheHost),
            };
        }
        self.app_context
            .rooms
            .mute(&self.request_context.room_id, &target_user_public_id)
            .await;
        let room_sockets_ids = self.app_context.rooms.all_socket_ids(&self.request_context.room_id).await;
        let ws_event_msg = SocketMessage {
            r#type: SocketMessageType::UserMuted,
            payload: None,
        };
        let raw_ws_event_msg = serde_json::to_string(&ws_event_msg).unwrap();
        self.app_context
            .sockets
            .broadcast_msg(&raw_ws_event_msg, &room_sockets_ids)
            .await;
        MuteUserResponse {
            error: false,
            error_code: None,
        }
    }

    pub async fn unmute(&self, target_user_public_id: String) -> UnmuteUserResponse {
        if !self.app_context.rooms.exists(&self.request_context.room_id).await {
            return UnmuteUserResponse {
                error: true,
                error_code: Some(UserUnmutingError::RoomNotFound),
            };
        }
        if !self
            .app_context
            .rooms
            .user_is_host(&self.request_context.room_id, &self.request_context.public_id)
            .await
        {
            return UnmuteUserResponse {
                error: true,
                error_code: Some(UserUnmutingError::YouAreNotTheHost),
            };
        }
        self.app_context
            .rooms
            .unmute(&self.request_context.room_id, &target_user_public_id)
            .await;
        let room_sockets_ids = self.app_context.rooms.all_socket_ids(&self.request_context.room_id).await;
        let ws_event_msg = SocketMessage {
            r#type: SocketMessageType::UserUnmuted,
            payload: None,
        };
        let raw_ws_event_msg = serde_json::to_string(&ws_event_msg).unwrap();
        self.app_context
            .sockets
            .broadcast_msg(&raw_ws_event_msg, &room_sockets_ids)
            .await;
        UnmuteUserResponse {
            error: false,
            error_code: None,
        }
    }

    pub async fn ban(&self, target_user_public_id: String) -> BanUserResponse {
        if !self.app_context.rooms.exists(&self.request_context.room_id).await {
            return BanUserResponse {
                error: true,
                error_code: Some(UserBanningError::RoomNotFound),
            };
        }
        if !self
            .app_context
            .rooms
            .user_is_host(&self.request_context.room_id, &self.request_context.public_id)
            .await
        {
            return BanUserResponse {
                error: true,
                error_code: Some(UserBanningError::YouAreNotTheHost),
            };
        }
        let room_sockets_ids = self.app_context.rooms.all_socket_ids(&self.request_context.room_id).await;
        self.app_context
            .rooms
            .ban(&self.request_context.room_id, &target_user_public_id)
            .await;
        let ws_event_msg = SocketMessage {
            r#type: SocketMessageType::UserBanned,
            payload: Some(SocketMessagePayload::Username(UserPubIdInfoPayload {
                // TODO: update code from username to id
                public_id: target_user_public_id,
            })),
        };
        let raw_ws_event_msg = serde_json::to_string(&ws_event_msg).unwrap();
        self.app_context
            .sockets
            .broadcast_msg(&raw_ws_event_msg, &room_sockets_ids)
            .await;
        BanUserResponse {
            error: false,
            error_code: None,
        }
    }

    pub async fn change_score(&self, target_user_public_id: String, amount: i64) -> ChangeScoreResponse {
        if !self.app_context.rooms.exists(&self.request_context.room_id).await {
            return ChangeScoreResponse {
                error: true,
                error_code: Some(ScoreChangeError::RoomNotFound),
            };
        }
        if !self
            .app_context
            .rooms
            .user_is_host(&self.request_context.room_id, &self.request_context.public_id)
            .await
        {
            return ChangeScoreResponse {
                error: true,
                error_code: Some(ScoreChangeError::YouAreNotTheHost),
            };
        }
        let room_sockets_ids = self.app_context.rooms.all_socket_ids(&self.request_context.room_id).await;
        self.app_context
            .rooms
            .change_score(&self.request_context.room_id, &target_user_public_id, amount)
            .await;
        let msg = "{\"type\": \"userScoreChanged\", \"payload\": null}".to_string();
        self.app_context
            .sockets
            .broadcast_msg(&msg, &room_sockets_ids)
            .await;
        ChangeScoreResponse {
            error: false,
            error_code: None,
        }
    }
}
