use crate::app_context::AppContext;
use crate::rooms::consts::MAX_USERNAME_LENGTH;
use std::convert::Infallible;
use unicode_segmentation::UnicodeSegmentation;

pub struct RoomHttpHandler {
    app_context: AppContext,
    room_id: String,
    user_id: String,
}

impl RoomHttpHandler {
    pub fn new(app_context: AppContext, room_id: String, user_id: String) -> Self {
        Self {
            app_context,
            room_id,
            user_id,
        }
    }

    pub async fn can_connect(&self, username: String) -> Result<String, Infallible> {
        if !self.app_context.rooms.such_room_exists(&self.room_id).await {
            return Ok::<_, Infallible>(
                "{\"canConnect\": false, \"reason\": \"Room not found.\"}".to_string(),
            );
        }
        if self
            .app_context
            .rooms
            .room_has_user_with_such_username(&self.room_id, &username, &self.user_id)
            .await
        {
            return Ok::<_, Infallible>(
                "{\"canConnect\": false, \"reason\": \"Such user already in the room.\"}"
                    .to_string(),
            );
        }
        if self
            .app_context
            .rooms
            .user_is_banned(&self.room_id, &self.user_id)
            .await
        {
            return Ok::<_, Infallible>(
                "{\"canConnect\": false, \"reason\": \"User is banned.\"}".to_string(),
            );
        }
        if username.graphemes(true).count() > MAX_USERNAME_LENGTH {
            eprintln!(
                "Rejecting user access to a room because the username is too long: \
                {} symbols when at most {} is allowed.",
                username.len(),
                MAX_USERNAME_LENGTH,
            );
            return Ok::<_, Infallible>(
                "{\"canConnect\": false, \"reason\": \"The username is too long.\"}".to_string(),
            );
        }
        Ok::<_, Infallible>("{\"canConnect\": true}".to_string())
    }

    pub async fn users(&self) -> Result<String, Infallible> {
        if !self.app_context.rooms.such_room_exists(&self.room_id).await {
            return Ok::<_, Infallible>(
                "{\"error\": true, \"reason\": \"Room not found.\"}".to_string(),
            );
        }
        Ok::<_, Infallible>(format!(
            "{{\"error\": false, \"users\": {}, \"status\": {}}}",
            self.app_context
                .rooms
                .users_of_room_as_json(&self.room_id)
                .await,
            self.app_context
                .rooms
                .room_status_as_json(&self.room_id)
                .await,
        ))
    }

    pub async fn messages(&self) -> Result<String, Infallible> {
        if !self.app_context.rooms.such_room_exists(&self.room_id).await {
            return Ok::<_, Infallible>(
                "{\"error\": true, \"reason\": \"Room not found.\"}".to_string(),
            );
        }
        Ok::<_, Infallible>(format!(
            "{{\"error\": false, \"messages\": {}}}",
            self.app_context
                .rooms
                .room_messages_as_json(&self.room_id)
                .await,
        ))
    }
}

pub struct CreateRoomHttpHandler {
    app_context: AppContext,
}

impl CreateRoomHttpHandler {
    pub fn new(app_context: AppContext) -> Self {
        Self { app_context }
    }

    pub async fn create(&self) -> Result<String, Infallible> {
        let room_id = self.app_context.rooms.create_room().await;
        Ok::<_, Infallible>(format!("{{\"roomId\": \"{}\"}}", room_id))
    }
}
