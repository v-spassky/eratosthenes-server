use crate::map_locations::models::LatLng;
use crate::rooms::message_types::BriefUserInfoPayload;
use crate::rooms::models::ChatMessage;

use crate::storage::rooms::UserConnectedResult;
use crate::storage::sockets::HashMapClientSocketsStorage;

pub trait IRoomStorage:
    RoomRepo
    + RoomGameFlowHandler
    + RoomConnectionHandler
    + RoomSocketsRepo
    + UserScoreRepo
    + UserGuessRepo
    + UserPermissionsRepo
    + RoomInfoSerializer
{
}

pub trait RoomRepo {
    async fn exists(&self, room_id: &str) -> bool;

    async fn create(&self) -> String;

    async fn has_user_with_such_username(
        &self,
        room_id: &str,
        username: &str,
        user_id: &str,
    ) -> bool;

    async fn has_user_with_such_id(&self, room_id: &str, user_id: &str) -> bool;

    async fn user_is_host(&self, room_id: &str, user_id: &str) -> bool;

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
        user_id: &str,
    ) -> Result<UserConnectedResult, ()>;

    async fn on_user_reconnected(
        &self,
        room_id: &str,
        _msg_payload: BriefUserInfoPayload,
        socket_id: usize,
        user_id: &str,
    );

    async fn on_user_disconnected(
        &self,
        room_id: &str,
        raw_msg: String,
        user_id: &str,
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
    async fn change_score(&self, room_id: &str, username: &str, amount: i64);
}

pub trait UserGuessRepo {
    async fn submit_guess(&self, room_id: &str, user_id: &str, guess: LatLng) -> bool;

    async fn revoke_guess(&self, room_id: &str, user_id: &str);
}

pub trait UserPermissionsRepo {
    async fn mute(&self, room_id: &str, user_id_to_mute: &str);

    async fn unmute(&self, room_id: &str, user_id_to_unmute: &str);

    async fn ban(&self, room_id: &str, user_name_to_ban: &str);

    async fn is_muted(&self, room_id: &str, user_id: &str) -> bool;

    async fn is_banned(&self, room_id: &str, user_id: &str) -> bool;
}

pub trait RoomInfoSerializer {
    async fn status_as_json(&self, room_id: &str) -> String;

    async fn users_as_json(&self, room_id: &str) -> String;

    async fn messages_as_json(&self, room_id: &str) -> String;
}
