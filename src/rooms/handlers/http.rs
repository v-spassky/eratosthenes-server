use crate::app_context::{AppContext, RequestContext};
use crate::rooms::consts::MAX_USERNAME_LENGTH;
use crate::rooms::handlers::responses::{
    CanConnectToRoomResponse, ConnectionRefusalError, CreateRoomResponse, RoomMessagesResponse,
    RoomMessagesResponseError, RoomUsersResponse, RoomUsersResponseError,
};
use crate::storage::interface::IRoomStorage;
use unicode_segmentation::UnicodeSegmentation;

pub struct RoomHttpHandler<'a, RS: IRoomStorage> {
    app_context: AppContext<RS>,
    request_context: &'a RequestContext,
}

impl<'a, RS> RoomHttpHandler<'a, RS>
where
    RS: IRoomStorage,
{
    pub fn new(app_context: AppContext<RS>, request_context: &'a RequestContext) -> Self {
        Self {
            app_context,
            request_context,
        }
    }

    pub async fn can_connect(&self, username: String) -> CanConnectToRoomResponse {
        if !self
            .app_context
            .rooms
            .exists(&self.request_context.room_id)
            .await
        {
            return CanConnectToRoomResponse {
                can_connect: false,
                error_code: Some(ConnectionRefusalError::RoomNotFound),
            };
        }
        if self
            .app_context
            .rooms
            .has_different_user_with_same_username(
                &self.request_context.room_id,
                &self.request_context.public_id,
                &username,
            )
            .await
        {
            return CanConnectToRoomResponse {
                can_connect: false,
                error_code: Some(ConnectionRefusalError::UserAlreadyInRoom),
            };
        }
        if self
            .app_context
            .rooms
            .is_banned(
                &self.request_context.room_id,
                &self.request_context.public_id,
            )
            .await
        {
            return CanConnectToRoomResponse {
                can_connect: false,
                error_code: Some(ConnectionRefusalError::UserBanned),
            };
        }
        if username.graphemes(true).count() > MAX_USERNAME_LENGTH {
            eprintln!(
                "Rejecting user access to a room because the username is too long: \
                {} symbols when at most {} is allowed.",
                username.len(),
                MAX_USERNAME_LENGTH,
            );
            return CanConnectToRoomResponse {
                can_connect: false,
                error_code: Some(ConnectionRefusalError::UsernameTooLong),
            };
        }
        CanConnectToRoomResponse {
            can_connect: true,
            error_code: None,
        }
    }

    pub async fn users(&self) -> RoomUsersResponse {
        if !self
            .app_context
            .rooms
            .exists(&self.request_context.room_id)
            .await
        {
            return RoomUsersResponse {
                error: true,
                error_code: Some(RoomUsersResponseError::RoomNotFound),
                users: None,
                status: None,
            };
        }
        RoomUsersResponse {
            error: false,
            error_code: None,
            users: Some(
                self.app_context
                    .rooms
                    .users(&self.request_context.room_id)
                    .await,
            ),
            status: Some(
                self.app_context
                    .rooms
                    .status(&self.request_context.room_id)
                    .await,
            ),
        }
    }

    pub async fn messages(&self) -> RoomMessagesResponse {
        if !self
            .app_context
            .rooms
            .exists(&self.request_context.room_id)
            .await
        {
            return RoomMessagesResponse {
                error: true,
                error_code: Some(RoomMessagesResponseError::RoomNotFound),
                messages: None,
            };
        }
        RoomMessagesResponse {
            error: false,
            error_code: None,
            messages: Some(
                self.app_context
                    .rooms
                    .messages(&self.request_context.room_id)
                    .await,
            ),
        }
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

    pub async fn create(&self) -> CreateRoomResponse {
        CreateRoomResponse {
            room_id: self.app_context.rooms.create().await,
        }
    }
}
