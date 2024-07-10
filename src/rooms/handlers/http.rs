use crate::app_context::AppContext;
use crate::rooms::consts::MAX_USERNAME_LENGTH;
use crate::storage::interface::IRoomStorage;
use std::convert::Infallible;
use unicode_segmentation::UnicodeSegmentation;

pub struct RoomHttpHandler<RS: IRoomStorage> {
    app_context: AppContext<RS>,
    room_id: String,
    user_id: String,
}

impl<RS> RoomHttpHandler<RS>
where
    RS: IRoomStorage,
{
    pub fn new(app_context: AppContext<RS>, room_id: String, user_id: String) -> Self {
        Self {
            app_context,
            room_id,
            user_id,
        }
    }

    pub async fn can_connect(&self, username: String) -> Result<String, Infallible> {
        if !self.app_context.rooms.exists(&self.room_id).await {
            return Ok::<_, Infallible>(
                "{\"canConnect\": false, \"reason\": \"Room not found.\"}".to_string(),
            );
        }
        if self
            .app_context
            .rooms
            .has_user_with_such_username(&self.room_id, &username, &self.user_id)
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
            .is_banned(&self.room_id, &self.user_id)
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
        if !self.app_context.rooms.exists(&self.room_id).await {
            return Ok::<_, Infallible>(
                "{\"error\": true, \"reason\": \"Room not found.\"}".to_string(),
            );
        }
        Ok::<_, Infallible>(format!(
            "{{\"error\": false, \"users\": {}, \"status\": {}}}",
            self.app_context.rooms.users_as_json(&self.room_id).await,
            self.app_context.rooms.status_as_json(&self.room_id).await,
        ))
    }

    pub async fn messages(&self) -> Result<String, Infallible> {
        if !self.app_context.rooms.exists(&self.room_id).await {
            return Ok::<_, Infallible>(
                "{\"error\": true, \"reason\": \"Room not found.\"}".to_string(),
            );
        }
        Ok::<_, Infallible>(format!(
            "{{\"error\": false, \"messages\": {}}}",
            self.app_context.rooms.messages_as_json(&self.room_id).await,
        ))
    }
}

pub struct CreateRoomHttpHandler<RS: IRoomStorage> {
    app_context: AppContext<RS>,
}

impl<RS> CreateRoomHttpHandler<RS>
where
    RS: IRoomStorage,
{
    pub fn new(app_context: AppContext<RS>) -> Self {
        Self { app_context }
    }

    pub async fn create(&self) -> Result<String, Infallible> {
        let room_id = self.app_context.rooms.create().await;
        Ok::<_, Infallible>(format!("{{\"roomId\": \"{}\"}}", room_id))
    }
}
