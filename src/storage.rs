use crate::message_types::{BriefUserInfoPayload, ChatMessagePayload};
use crate::models::{LatLng, Room, RoomStatus, User};
use rand::{distributions::Alphanumeric, Rng};
use std::collections::HashMap;
use std::collections::VecDeque;
use std::sync::{atomic::AtomicUsize, Arc};
use std::time::Duration;
use tokio::sync::{mpsc, RwLock};
use warp::ws::Message;

pub static NEXT_USER_ID: AtomicUsize = AtomicUsize::new(1);
pub const HOW_MUCH_LAST_MESSAGES_TO_STORE: usize = 30;

pub type ClientSockets = Arc<RwLock<HashMap<usize, mpsc::UnboundedSender<Message>>>>;

#[derive(Clone, Default)]
pub struct Rooms {
    storage: Arc<RwLock<HashMap<String, Room>>>,
}

impl Rooms {
    pub async fn such_room_exists(&self, room_id: &str) -> bool {
        self.storage.read().await.contains_key(room_id)
    }

    pub async fn room_has_user_with_such_username(
        &self,
        room_id: &str,
        username: &str,
        user_id: &str,
    ) -> bool {
        self.storage
            .read()
            .await
            .get(room_id)
            .unwrap()
            .users
            .iter()
            .any(|user| user.name == username && user.id != user_id)
    }

    pub async fn user_is_host_of_the_room(&self, room_id: &str, user_id: &str) -> bool {
        self.storage
            .read()
            .await
            .get(room_id)
            .unwrap()
            .users
            .iter()
            .find(|user| user.id == user_id)
            .map_or(false, |user| user.is_host)
    }

    pub async fn user_is_muted(&self, room_id: &str, user_id: &str) -> bool {
        self.storage
            .read()
            .await
            .get(room_id)
            .unwrap()
            .users
            .iter()
            .find(|user| user.id == user_id)
            .map_or(false, |user| user.is_muted)
    }

    pub async fn user_is_banned(&self, room_id: &str, user_id: &String) -> bool {
        self.storage
            .read()
            .await
            .get(room_id)
            .unwrap()
            .banned_users_ids
            .contains(user_id)
    }

    pub async fn users_of_room_as_json(&self, room_id: &str) -> String {
        self.storage
            .read()
            .await
            .get(room_id)
            .unwrap()
            .users_as_json()
    }

    pub async fn room_messages_as_json(&self, room_id: &str) -> String {
        self.storage
            .read()
            .await
            .get(room_id)
            .unwrap()
            .messages_as_json()
    }

    pub async fn room_status_as_json(&self, room_id: &str) -> String {
        self.storage
            .read()
            .await
            .get(room_id)
            .unwrap()
            .status
            .as_json()
    }

    pub async fn submit_user_guess(&self, room_id: &str, user_id: &str, guess: LatLng) -> bool {
        let mut storage_guard = self.storage.write().await;
        let room = storage_guard.get_mut(room_id).unwrap();
        room.users
            .iter_mut()
            .find(|user| user.id == *user_id)
            .unwrap()
            .submit_guess(guess, room.status);
        room.users.iter().all(|user| user.submitted_guess)
    }

    pub async fn revoke_user_guess(&self, room_id: &str, user_id: &str) {
        self.storage
            .write()
            .await
            .get_mut(room_id)
            .unwrap()
            .users
            .iter_mut()
            .find(|user| user.id == *user_id)
            .unwrap()
            .revoke_guess();
    }

    pub async fn mute_user(&self, room_id: &str, user_id_to_mute: &str) {
        self.storage
            .write()
            .await
            .get_mut(room_id)
            .unwrap()
            .users
            .iter_mut()
            .find(|user| user.name == *user_id_to_mute)
            .unwrap()
            .mute();
    }

    pub async fn unmute_user(&self, room_id: &str, user_id_to_unmute: &str) {
        self.storage
            .write()
            .await
            .get_mut(room_id)
            .unwrap()
            .users
            .iter_mut()
            .find(|user| user.name == *user_id_to_unmute)
            .unwrap()
            .unmute();
    }

    pub async fn ban_user(&self, room_id: &str, user_name_to_ban: &str) {
        self.storage
            .write()
            .await
            .get_mut(room_id)
            .unwrap()
            .ban_user(user_name_to_ban)
    }

    pub async fn change_user_score(&self, room_id: &str, username: &str, amount: i64) {
        self.storage
            .write()
            .await
            .get_mut(room_id)
            .unwrap()
            .users
            .iter_mut()
            .find(|user| user.name == *username)
            .unwrap()
            .change_score(amount)
    }

    pub async fn create_room(&self) -> String {
        let room_id = generate_room_id();
        let room = Room {
            users: vec![],
            last_messages: VecDeque::with_capacity(HOW_MUCH_LAST_MESSAGES_TO_STORE),
            status: RoomStatus::Waiting {
                previous_location: None,
            },
            banned_users_ids: vec![],
        };
        self.storage.write().await.insert(room_id.clone(), room);
        room_id
    }

    pub async fn handle_new_user_connected(
        &self,
        room_id: &str,
        msg_payload: BriefUserInfoPayload,
        socket_id: usize,
        user_id: &str,
    ) -> Result<UserConnectedResult, ()> {
        let mut storage_guard = self.storage.write().await;
        let room_has_no_members = storage_guard.get(room_id).unwrap().users.is_empty();
        let description_ids_of_room_members = storage_guard
            .get(room_id)
            .unwrap()
            .users
            .iter()
            .map(|user| user.description_id)
            .collect::<Vec<_>>();
        let such_user_already_in_the_room = storage_guard
            .get(room_id)
            .unwrap()
            .users
            .iter()
            .any(|user| user.id == user_id);
        if such_user_already_in_the_room {
            storage_guard
                .get_mut(room_id)
                .unwrap()
                .users
                .iter_mut()
                .find(|user| user.id == user_id)
                .unwrap()
                .socket_id = Some(socket_id);
            // TODO: comparison by user ID, not by usernames - return Err if exists
            return Ok(UserConnectedResult::AlreadyInTheRoom);
        }
        storage_guard
            .get_mut(room_id)
            .unwrap()
            .users
            .push(User::new(
                user_id.to_string(),
                msg_payload.username,
                msg_payload.avatar_emoji,
                room_has_no_members,
                description_ids_of_room_members,
                socket_id,
            ));
        Ok(UserConnectedResult::NewUser)
    }

    pub async fn handle_user_reconnected(
        &self,
        room_id: &str,
        _msg_payload: BriefUserInfoPayload,
        socket_id: usize,
        user_id: &str,
    ) {
        self.storage
            .write()
            .await
            .get_mut(room_id)
            .unwrap()
            .users
            .iter_mut()
            .find(|user| user.id == user_id)
            .unwrap()
            .socket_id = Some(socket_id);
    }

    pub async fn handle_user_disconnected(
        &self,
        room_id: &str,
        raw_msg: String,
        user_id: &str,
        socket_id: usize,
        client_sockets: ClientSockets,
    ) {
        let storage_handle = self.storage.clone();
        let room_id = room_id.to_string();
        let user_id = user_id.to_string();
        let relevant_socket_ids = self.relevant_socket_ids(&room_id, socket_id).await;
        tokio::spawn(async move {
            println!("[handle_user_disconnected]: waiting before running...");
            tokio::time::sleep(Duration::from_secs(5)).await;
            println!("[handle_user_disconnected]: running");
            let mut storage_guard = storage_handle.write().await;
            let such_user_already_in_the_room = storage_guard
                .get(&room_id)
                .unwrap()
                .users
                .iter()
                .any(|user| user.id == user_id && user.socket_id.is_some());
            if such_user_already_in_the_room {
                println!("[handle_user_disconnected]: {user_id} already in the room");
                return;
            }
            let user_is_host = storage_guard
                .get(&room_id)
                .unwrap()
                .users
                .iter()
                .find(|user| user.id == user_id)
                .unwrap()
                .is_host;
            storage_guard
                .get_mut(&room_id)
                .unwrap()
                .users
                .retain(|user| user.id != user_id);
            if user_is_host {
                storage_guard.get_mut(&room_id).unwrap().reassign_host()
            }

            println!(
                "[user_message]: broadcasting message {raw_msg} to users: {relevant_socket_ids:?}"
            );
            for (&uid, tx) in client_sockets.read().await.iter() {
                if relevant_socket_ids.contains(&Some(uid)) {
                    if let Err(_disconnected) = tx.send(Message::text(raw_msg.clone())) {
                        // The tx is disconnected, our `user_disconnected` code
                        // should be happening in another task, nothing more to
                        // do here.
                        eprintln!(
                            "[user_message]: error broadcasting message {raw_msg} to user ith id {uid:?}"
                        );
                    }
                }
            }
        });
    }

    pub async fn disconnect_user(&self, room_id: &str, socket_id: usize) {
        let mut storage_guard = self.storage.write().await;
        let user = storage_guard
            .get_mut(room_id)
            .unwrap()
            .users
            .iter_mut()
            .find(|user| user.socket_id == Some(socket_id));
        match user {
            Some(user) => {
                // socket closed not on behalf of the user
                user.socket_id = None;
            }
            None => {
                println!("[user_disconnected]: user with such socket id not found: {socket_id}");
            }
        }
    }

    pub async fn finish_game(&self, room_id: &str) {
        self.storage
            .write()
            .await
            .get_mut(room_id)
            .unwrap()
            .finish_game();
    }

    pub async fn handle_game_started(&self, room_id: &str, client_sockets: ClientSockets) {
        self.storage
            .write()
            .await
            .get_mut(room_id)
            .unwrap()
            .start_playing();
        let room_id = room_id.to_string();
        let storage_handle = self.storage.clone();
        tokio::spawn(async move {
            for tick in (0..=100).rev() {
                let all_sockets_ids = storage_handle
                    .read()
                    .await
                    .get(&room_id)
                    .unwrap()
                    .users
                    .iter()
                    .map(|user| user.socket_id)
                    .collect::<Vec<_>>();
                let raw_msg = format!("{{\"type\":\"tick\",\"payload\":{tick}}}");
                tokio::time::sleep(Duration::from_secs(1)).await;
                // Check if the game was finished because all players submitted a guess
                // before the timer counted all the way down
                let room_status = storage_handle.read().await.get(&room_id).unwrap().status;
                if let RoomStatus::Waiting {
                    previous_location: _previous_location,
                } = room_status
                {
                    return;
                }
                println!(
                    "[user_message]: broadcasting message {raw_msg} to users: {all_sockets_ids:?}"
                );
                for (&uid, tx) in client_sockets.read().await.iter() {
                    if all_sockets_ids.contains(&Some(uid)) {
                        if let Err(_disconnected) = tx.send(Message::text(raw_msg.clone())) {
                            // The tx is disconnected, our `user_disconnected` code
                            // should be happening in another task, nothing more to
                            // do here.
                            eprintln!(
                                "[user_message]: error broadcasting message {raw_msg} to user ith id {uid:?}"
                            );
                        }
                    }
                }
            }
            storage_handle
                .write()
                .await
                .get_mut(&room_id)
                .unwrap()
                .finish_game();
            let all_sockets_ids = storage_handle
                .read()
                .await
                .get(&room_id)
                .unwrap()
                .users
                .iter()
                .map(|user| user.socket_id)
                .collect::<Vec<_>>();
            let finish_msg = "{\"type\":\"gameFinished\",\"payload\":null}".to_string();
            for (&uid, tx) in client_sockets.read().await.iter() {
                if all_sockets_ids.contains(&Some(uid)) {
                    if let Err(_disconnected) = tx.send(Message::text(finish_msg.clone())) {
                        // The tx is disconnected, our `user_disconnected` code
                        // should be happening in another task, nothing more to
                        // do here.
                        eprintln!(
                            "[user_message]: error broadcasting message {finish_msg} to user ith id {uid:?}"
                        );
                    }
                }
            }
        });
    }

    pub async fn relevant_socket_ids(&self, room_id: &str, socket_id: usize) -> Vec<Option<usize>> {
        self.storage
            .read()
            .await
            .get(room_id)
            .unwrap()
            .users
            .iter()
            .filter(|user| user.socket_id != Some(socket_id)) // TODO: maybe compare by username ?
            .map(|user| user.socket_id)
            .collect::<Vec<_>>()
    }

    pub async fn all_socket_ids(&self, room_id: &str) -> Vec<Option<usize>> {
        self.storage
            .read()
            .await
            .get(room_id)
            .unwrap()
            .users
            .iter()
            .map(|user| user.socket_id)
            .collect::<Vec<_>>()
    }

    pub async fn add_new_message(&self, room_id: &str, message: ChatMessagePayload) {
        self.storage
            .write()
            .await
            .get_mut(room_id)
            .unwrap()
            .add_message(message.to_model());
    }
}

fn generate_room_id() -> String {
    rand::thread_rng()
        .sample_iter(&Alphanumeric)
        .take(10)
        .map(char::from)
        .collect()
}

pub enum UserConnectedResult {
    NewUser,
    AlreadyInTheRoom,
}
