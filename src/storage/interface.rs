use crate::map_locations::models::LatLng;
use crate::rooms::message_types::BriefUserInfoPayload;
use crate::rooms::models::{ChatMessage, RoomStatus};

use crate::storage::rooms::UserConnectedResult;
use crate::storage::sockets::HashMapClientSocketsStorage;
use crate::users::models::User;

pub trait IRoomStorage:
    RoomRepo
    + RoomGameFlowHandler
    + RoomConnectionHandler
    + RoomSocketsRepo
    + UserScoreRepo
    + UserGuessRepo
    + UserPermissionsRepo
    + RoomInfoRepo
{
}

pub trait RoomRepo {
    async fn exists(&self, room_id: &str) -> bool;

    async fn create(&self) -> String;

    async fn has_different_user_with_same_username(
        &self,
        room_id: &str,
        public_user_id: &str,
        username: &str,
    ) -> bool;

    async fn has_user_with_such_private_id(&self, room_id: &str, private_user_id: &str) -> bool;

    async fn user_is_host(&self, room_id: &str, public_user_id: &str) -> bool;

    async fn add_message(&self, room_id: &str, message: ChatMessage);
}

pub trait RoomGameFlowHandler {
    async fn start_game(&self, room_id: &str, client_sockets: HashMapClientSocketsStorage);

    async fn finish_game(&self, room_id: &str) -> bool;

    async fn current_round_number(&self, room_id: &str) -> u64;
}

pub trait RoomConnectionHandler {
    async fn on_user_connected(
        &self,
        room_id: &str,
        msg_payload: BriefUserInfoPayload,
        socket_id: usize,
        public_user_id: &str,
        private_user_id: &str,
    ) -> Result<UserConnectedResult, ()>;

    async fn on_user_reconnected(
        &self,
        room_id: &str,
        _msg_payload: BriefUserInfoPayload,
        socket_id: usize,
        private_user_id: &str,
    );

    async fn on_user_disconnected(
        &self,
        room_id: &str,
        raw_msg: String,
        private_user_id: &str,
        socket_id: usize,
        client_sockets: HashMapClientSocketsStorage,
    );

    async fn disconnect_user(&self, room_id: &str, socket_id: usize);
}

pub trait RoomSocketsRepo {
    async fn all_socket_ids(&self, room_id: &str) -> Vec<Option<usize>>;

    async fn socket_ids_except_sender(
        &self,
        room_id: &str,
        sender_socket_id: usize,
    ) -> Vec<Option<usize>>;
}

pub trait UserScoreRepo {
    async fn change_score(&self, room_id: &str, target_user_public_id: &str, amount: i64);
}

pub trait UserGuessRepo {
    async fn save_guess(&self, room_id: &str, private_user_id: &str, guess: LatLng);

    async fn submit_guess(&self, room_id: &str, private_user_id: &str, guess: LatLng) -> bool;

    async fn revoke_guess(&self, room_id: &str, private_user_id: &str);
}

pub trait UserPermissionsRepo {
    async fn mute(&self, room_id: &str, target_user_public_id: &str);

    async fn unmute(&self, room_id: &str, target_user_public_id: &str);

    async fn ban(&self, room_id: &str, target_user_public_id: &str);

    async fn is_muted(&self, room_id: &str, public_user_id: &str) -> bool;

    async fn is_banned(&self, room_id: &str, public_user_id: &str) -> bool;
}

pub trait RoomInfoRepo {
    async fn status(&self, room_id: &str) -> RoomStatus;

    async fn users(&self, room_id: &str) -> Vec<User>;

    async fn messages(&self, room_id: &str) -> Vec<ChatMessage>;
}
