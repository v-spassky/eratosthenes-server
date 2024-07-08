use crate::app_context::AppContext;
use crate::map_locations::models::LatLng;
use crate::rooms::consts::ROUNDS_PER_GAME;
use crate::rooms::message_types::{SocketMessage, SocketMessageType};
use crate::rooms::models::ChatMessage;
use std::collections::HashMap;
use std::convert::Infallible;

pub struct UsersHttpHandler {
    app_context: AppContext,
    room_id: String,
    user_id: String,
}

impl UsersHttpHandler {
    pub fn new(app_context: AppContext, room_id: String, user_id: String) -> Self {
        Self {
            app_context,
            room_id,
            user_id,
        }
    }

    pub async fn is_host(&self) -> Result<String, Infallible> {
        if !self.app_context.rooms.such_room_exists(&self.room_id).await {
            return Ok::<_, Infallible>("{\"isHost\": false}".to_string());
        }
        Ok::<_, Infallible>(format!(
            "{{\"isHost\": {}}}",
            self.app_context
                .rooms
                .user_is_host_of_the_room(&self.room_id, &self.user_id)
                .await
        ))
    }

    pub async fn submit_guess(
        &self,
        guess_json: HashMap<String, String>,
    ) -> Result<String, Infallible> {
        if !self.app_context.rooms.such_room_exists(&self.room_id).await {
            return Ok::<_, Infallible>(
                "{\"error\": true, \"reason\": \"Room not found.\"}".to_string(),
            );
        }
        let guess = LatLng {
            lat: guess_json.get("lat").unwrap().parse().unwrap(),
            lng: guess_json.get("lng").unwrap().parse().unwrap(),
        };
        let round_finished = self
            .app_context
            .rooms
            .submit_user_guess(&self.room_id, &self.user_id, guess)
            .await;
        let room_sockets_ids = self.app_context.rooms.all_socket_ids(&self.room_id).await;
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
            let game_finished = self.app_context.rooms.finish_game(&self.room_id).await;
            let msg = if game_finished {
                "{\"type\":\"gameFinished\",\"payload\":null}".to_string()
            } else {
                "{\"type\":\"roundFinished\",\"payload\":null}".to_string()
            };
            let rounds_left = self
                .app_context
                .rooms
                .get_current_round_number(&self.room_id)
                .await;
            let round_number = match rounds_left {
                ROUNDS_PER_GAME => ROUNDS_PER_GAME,
                _ => ROUNDS_PER_GAME - rounds_left,
            };
            let bot_msg_content = format!("Раунд {round_number}/{ROUNDS_PER_GAME} закончился.");
            let bot_msg = format!(
                "{{\"type\": \"chatMessage\", \"payload\": {{\"from\": null,
                \"content\": \"{}\", \"isFromBot\": true}}}}",
                bot_msg_content,
            );
            let bot_message = ChatMessage {
                is_from_bot: true,
                author_name: None,
                content: bot_msg_content,
            };
            self.app_context
                .rooms
                .add_new_message(&self.room_id, bot_message)
                .await;
            self.app_context
                .sockets
                .broadcast_msg(&bot_msg, &room_sockets_ids)
                .await;
            self.app_context
                .sockets
                .broadcast_msg(&msg, &room_sockets_ids)
                .await;
        }
        Ok::<_, Infallible>(String::from("{\"error\": false}"))
    }

    pub async fn revoke_guess(&self) -> Result<String, Infallible> {
        if !self.app_context.rooms.such_room_exists(&self.room_id).await {
            return Ok::<_, Infallible>(
                "{\"error\": true, \"reason\": \"Room not found.\"}".to_string(),
            );
        }
        self.app_context
            .rooms
            .revoke_user_guess(&self.room_id, &self.user_id)
            .await;
        let room_sockets_ids = self.app_context.rooms.all_socket_ids(&self.room_id).await;
        let msg = SocketMessage {
            r#type: SocketMessageType::GuessRevoked,
            payload: None,
        };
        let msg = serde_json::to_string(&msg).unwrap();
        self.app_context
            .sockets
            .broadcast_msg(&msg, &room_sockets_ids)
            .await;
        Ok::<_, Infallible>(String::from("{\"error\": false}"))
    }

    pub async fn mute(&self, target_username: String) -> Result<String, Infallible> {
        if !self.app_context.rooms.such_room_exists(&self.room_id).await {
            return Ok::<_, Infallible>(
                "{\"error\": true, \"reason\": \"Room not found.\"}".to_string(),
            );
        }
        if !self
            .app_context
            .rooms
            .user_is_host_of_the_room(&self.room_id, &self.user_id)
            .await
        {
            return Ok::<_, Infallible>(
                "{\"error\": true, \"reason\": \"You are not the host.\"}".to_string(),
            );
        }
        self.app_context
            .rooms
            .mute_user(&self.room_id, &target_username)
            .await;
        let room_sockets_ids = self.app_context.rooms.all_socket_ids(&self.room_id).await;
        let msg = "{\"type\": \"userMuted\", \"payload\": null}".to_string();
        self.app_context
            .sockets
            .broadcast_msg(&msg, &room_sockets_ids)
            .await;
        Ok::<_, Infallible>(String::from("{\"error\": false}"))
    }

    pub async fn unmute(&self, target_username: String) -> Result<String, Infallible> {
        if !self.app_context.rooms.such_room_exists(&self.room_id).await {
            return Ok::<_, Infallible>(
                "{\"error\": true, \"reason\": \"Room not found.\"}".to_string(),
            );
        }
        if !self
            .app_context
            .rooms
            .user_is_host_of_the_room(&self.room_id, &self.user_id)
            .await
        {
            return Ok::<_, Infallible>(
                "{\"error\": true, \"reason\": \"You are not the host.\"}".to_string(),
            );
        }
        self.app_context
            .rooms
            .unmute_user(&self.room_id, &target_username)
            .await;
        let room_sockets_ids = self.app_context.rooms.all_socket_ids(&self.room_id).await;
        let msg = "{\"type\": \"userUnmuted\", \"payload\": null}".to_string();
        self.app_context
            .sockets
            .broadcast_msg(&msg, &room_sockets_ids)
            .await;
        Ok::<_, Infallible>(String::from("{\"error\": false}"))
    }

    pub async fn ban(&self, target_username: String) -> Result<String, Infallible> {
        if !self.app_context.rooms.such_room_exists(&self.room_id).await {
            return Ok::<_, Infallible>(
                "{\"error\": true, \"reason\": \"Room not found.\"}".to_string(),
            );
        }
        if !self
            .app_context
            .rooms
            .user_is_host_of_the_room(&self.room_id, &self.user_id)
            .await
        {
            return Ok::<_, Infallible>(
                "{\"error\": true, \"reason\": \"You are not the host.\"}".to_string(),
            );
        }
        let room_sockets_ids = self.app_context.rooms.all_socket_ids(&self.room_id).await;
        self.app_context
            .rooms
            .ban_user(&self.room_id, &target_username)
            .await;
        let msg = format!(
            "{{\"type\": \"userBanned\", \"payload\": {{\"username\": \"{}\"}}}}",
            target_username,
        );
        self.app_context
            .sockets
            .broadcast_msg(&msg, &room_sockets_ids)
            .await;
        Ok::<_, Infallible>(String::from("{\"error\": false}"))
    }

    pub async fn change_score(
        &self,
        target_username: String,
        request_body: HashMap<String, String>,
    ) -> Result<String, Infallible> {
        if !self.app_context.rooms.such_room_exists(&self.room_id).await {
            return Ok::<_, Infallible>(
                "{\"error\": true, \"reason\": \"Room not found.\"}".to_string(),
            );
        }
        if !self
            .app_context
            .rooms
            .user_is_host_of_the_room(&self.room_id, &self.user_id)
            .await
        {
            return Ok::<_, Infallible>(
                "{\"error\": true, \"reason\": \"You are not the host.\"}".to_string(),
            );
        }
        let amount = request_body.get("amount").unwrap().parse::<i64>().unwrap();
        let room_sockets_ids = self.app_context.rooms.all_socket_ids(&self.room_id).await;
        self.app_context
            .rooms
            .change_user_score(&self.room_id, &target_username, amount)
            .await;
        let msg = "{\"type\": \"userScoreChanged\", \"payload\": null}".to_string();
        self.app_context
            .sockets
            .broadcast_msg(&msg, &room_sockets_ids)
            .await;
        Ok::<_, Infallible>(String::from("{\"error\": false}"))
    }
}
