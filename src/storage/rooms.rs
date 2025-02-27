use crate::map::models::LatLng;
use crate::rooms::consts::ROUNDS_PER_GAME;
use crate::rooms::message_types::{
    self, BotMessagePayload, BriefUserInfoPayload, RoundStartedBotMessagePayload,
    RoundStartedBotMsg, ServerSentSocketMessage, UserDisconnectedBotMessagePayload,
    UserDisconnectedBotMsg,
};
use crate::rooms::models::{ChatMessage, Room, RoomStatus};
use crate::storage::consts::HOW_MUCH_LAST_MESSAGES_TO_STORE;
use crate::storage::interface::{
    IRoomStorage, RoomConnectionHandler, RoomGameFlowHandler, RoomInfoRepo, RoomRepo,
    RoomSocketsRepo, UserGuessRepo, UserPermissionsRepo, UserScoreRepo,
};
use crate::storage::sockets::HashMapClientSocketsStorage;
use crate::users::models::User;
use rand::{distributions::Alphanumeric, Rng};
use std::collections::HashMap;
use std::collections::VecDeque;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::RwLock;

#[derive(Clone, Default)]
pub struct HashMapRoomsStorage {
    storage: Arc<RwLock<HashMap<String, Room>>>,
}

impl IRoomStorage for HashMapRoomsStorage {}

impl RoomRepo for HashMapRoomsStorage {
    async fn exists(&self, room_id: &str) -> bool {
        self.storage.read().await.contains_key(room_id)
    }

    async fn create(&self) -> String {
        let room_id = generate_room_id();
        let room = Room {
            users: vec![],
            last_messages: VecDeque::with_capacity(HOW_MUCH_LAST_MESSAGES_TO_STORE),
            status: RoomStatus::Waiting {
                previous_location: None,
            },
            banned_public_users_ids: vec![],
            rounds_left: ROUNDS_PER_GAME,
        };
        self.storage.write().await.insert(room_id.clone(), room);
        room_id
    }

    async fn has_different_user_with_same_username(
        &self,
        room_id: &str,
        public_user_id: &str,
        username: &str,
    ) -> bool {
        self.storage
            .read()
            .await
            .get(room_id)
            .unwrap()
            .users
            .iter()
            .any(|user| user.public_id != public_user_id && user.name == username)
    }

    async fn has_user_with_such_private_id(&self, room_id: &str, private_user_id: &str) -> bool {
        self.storage
            .read()
            .await
            .get(room_id)
            .unwrap()
            .users
            .iter()
            .any(|user| user.private_id == private_user_id)
    }

    async fn user_is_host(&self, room_id: &str, public_user_id: &str) -> bool {
        self.storage
            .read()
            .await
            .get(room_id)
            .unwrap()
            .users
            .iter()
            .find(|user| user.public_id == public_user_id)
            .map_or(false, |user| user.is_host)
    }

    async fn add_message(&self, room_id: &str, message: ChatMessage) {
        self.storage
            .write()
            .await
            .get_mut(room_id)
            .unwrap()
            .add_message(message);
    }
}

impl RoomGameFlowHandler for HashMapRoomsStorage {
    async fn start_game(&self, room_id: &str, client_sockets: HashMapClientSocketsStorage) {
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
                let ws_event_msg = ServerSentSocketMessage::Tick {
                    r#type: message_types::Tick,
                    payload: tick,
                };
                let raw_ws_event_msg = serde_json::to_string(&ws_event_msg).unwrap();
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
                client_sockets
                    .broadcast_msg(&raw_ws_event_msg, &all_sockets_ids)
                    .await;
            }
            let game_finished = storage_handle
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
            let game_or_round_finished_msg = if game_finished {
                ServerSentSocketMessage::GameFinished {
                    r#type: message_types::GameFinished,
                }
            } else {
                ServerSentSocketMessage::RoundFinished {
                    r#type: message_types::RoundFinished,
                }
            };
            let raw_game_or_round_finished_msg =
                serde_json::to_string(&game_or_round_finished_msg).unwrap();
            // TODO: bad because duplicates the `self.get_current_round_number()` code
            let rounds_left = storage_handle
                .read()
                .await
                .get(&room_id)
                .unwrap()
                .rounds_left;
            let round_number = match rounds_left {
                ROUNDS_PER_GAME => ROUNDS_PER_GAME,
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
            let bot_ws_msg = ServerSentSocketMessage::BotMessage {
                r#type: message_types::BotMessage,
                id: bot_message.id(),
                payload: bot_message_payload,
            };
            let raw_bot_ws_msg = serde_json::to_string(&bot_ws_msg).unwrap();
            // TODO: bad because duplicates the `self.add_new_message()` code
            storage_handle
                .write()
                .await
                .get_mut(&room_id)
                .unwrap()
                .add_message(bot_message);
            client_sockets
                .broadcast_msg(&raw_bot_ws_msg, &all_sockets_ids)
                .await;
            client_sockets
                .broadcast_msg(&raw_game_or_round_finished_msg, &all_sockets_ids)
                .await;
        });
    }

    async fn finish_game(&self, room_id: &str) -> bool {
        self.storage
            .write()
            .await
            .get_mut(room_id)
            .unwrap()
            .finish_game()
    }
    async fn current_round_number(&self, room_id: &str) -> u64 {
        self.storage.read().await.get(room_id).unwrap().rounds_left
    }
}

impl RoomConnectionHandler for HashMapRoomsStorage {
    async fn on_user_connected(
        &self,
        room_id: &str,
        msg_payload: BriefUserInfoPayload,
        socket_id: usize,
        public_user_id: &str,
        private_user_id: &str,
    ) -> Result<UserConnectedResult, ()> {
        let mut storage_guard = self.storage.write().await;
        let room_has_no_members = storage_guard.get(room_id).unwrap().users.is_empty();
        let such_user_already_in_the_room = storage_guard
            .get(room_id)
            .unwrap()
            .users
            .iter()
            .any(|user| user.private_id == private_user_id);
        if such_user_already_in_the_room {
            storage_guard
                .get_mut(room_id)
                .unwrap()
                .users
                .iter_mut()
                .find(|user| user.private_id == private_user_id)
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
                public_user_id.to_string(),
                private_user_id.to_string(),
                msg_payload.username,
                msg_payload.avatar_emoji,
                room_has_no_members,
                socket_id,
            ));
        Ok(UserConnectedResult::NewUser)
    }

    async fn on_user_reconnected(
        &self,
        room_id: &str,
        _msg_payload: BriefUserInfoPayload,
        socket_id: usize,
        private_user_id: &str,
    ) {
        self.storage
            .write()
            .await
            .get_mut(room_id)
            .unwrap()
            .users
            .iter_mut()
            .find(|user| user.private_id == private_user_id)
            .unwrap()
            .socket_id = Some(socket_id);
    }

    async fn on_user_disconnected(
        &self,
        room_id: &str,
        raw_msg: String,
        private_user_id: &str,
        socket_id: usize,
        client_sockets: HashMapClientSocketsStorage,
    ) {
        let storage_handle = self.storage.clone();
        let room_id = room_id.to_string();
        let private_user_id = private_user_id.to_string();
        let relevant_socket_ids = self.socket_ids_except_sender(&room_id, socket_id).await;
        tokio::spawn(async move {
            println!("[handle_user_disconnected]: waiting before disconnecting user...");
            tokio::time::sleep(Duration::from_secs(5)).await;
            println!("[handle_user_disconnected]: disconnecting user");
            let mut storage_guard = storage_handle.write().await;
            let such_user_already_in_the_room = storage_guard
                .get(&room_id)
                .unwrap()
                .users
                .iter()
                .any(|user| user.private_id == private_user_id && user.socket_id.is_some());
            if such_user_already_in_the_room {
                return;
            }
            let (index_of_user_to_remove, _user) = storage_guard
                .get(&room_id)
                .unwrap()
                .users
                .iter()
                .enumerate()
                .find(|(_idx, user)| user.private_id == private_user_id)
                .unwrap();
            let removed_user = storage_guard
                .get_mut(&room_id)
                .unwrap()
                .users
                .remove(index_of_user_to_remove);
            if removed_user.is_host {
                storage_guard.get_mut(&room_id).unwrap().reassign_host()
            }
            let bot_message_payload = BotMessagePayload::UserDisconnected {
                r#type: UserDisconnectedBotMsg,
                payload: UserDisconnectedBotMessagePayload {
                    username: removed_user.name,
                },
            };
            let bot_message = ChatMessage::from_bot(bot_message_payload.clone());
            let ws_message = ServerSentSocketMessage::BotMessage {
                r#type: message_types::BotMessage,
                id: bot_message.id(),
                payload: bot_message_payload,
            };
            let bot_message_content = serde_json::to_string(&ws_message).unwrap();
            let mut all_sockets_ids = relevant_socket_ids.clone();
            all_sockets_ids.push(Some(socket_id));
            storage_guard
                .get_mut(&room_id)
                .unwrap()
                .add_message(bot_message);
            client_sockets
                .broadcast_msg(&bot_message_content, &all_sockets_ids)
                .await;
            client_sockets
                .broadcast_msg(&raw_msg, &relevant_socket_ids)
                .await;
        });
    }

    async fn disconnect_user(&self, room_id: &str, socket_id: usize) {
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
}

impl RoomSocketsRepo for HashMapRoomsStorage {
    async fn all_socket_ids(&self, room_id: &str) -> Vec<Option<usize>> {
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

    async fn socket_ids_except_sender(
        &self,
        room_id: &str,
        sender_socket_id: usize,
    ) -> Vec<Option<usize>> {
        self.storage
            .read()
            .await
            .get(room_id)
            .unwrap()
            .users
            .iter()
            // TODO: maybe compare by username ?
            .filter(|user| user.socket_id != Some(sender_socket_id))
            .map(|user| user.socket_id)
            .collect::<Vec<_>>()
    }
}

impl UserScoreRepo for HashMapRoomsStorage {
    async fn change_score(&self, room_id: &str, target_user_public_id: &str, amount: i64) {
        self.storage
            .write()
            .await
            .get_mut(room_id)
            .unwrap()
            .users
            .iter_mut()
            .find(|user| user.public_id == *target_user_public_id)
            .unwrap()
            .change_score(amount)
    }
}

impl UserGuessRepo for HashMapRoomsStorage {
    async fn save_guess(&self, room_id: &str, private_user_id: &str, guess: LatLng) {
        let mut storage_guard = self.storage.write().await;
        let room = storage_guard.get_mut(room_id).unwrap();
        room.users
            .iter_mut()
            .find(|user| user.private_id == *private_user_id)
            .unwrap()
            .save_guess(guess);
    }

    async fn submit_guess(&self, room_id: &str, private_user_id: &str, guess: LatLng) -> bool {
        let mut storage_guard = self.storage.write().await;
        let room = storage_guard.get_mut(room_id).unwrap();
        room.users
            .iter_mut()
            .find(|user| user.private_id == *private_user_id)
            .unwrap()
            .submit_guess(guess, room.status);
        room.users.iter().all(|user| user.submitted_guess)
    }

    async fn revoke_guess(&self, room_id: &str, private_user_id: &str) {
        self.storage
            .write()
            .await
            .get_mut(room_id)
            .unwrap()
            .users
            .iter_mut()
            .find(|user| user.private_id == *private_user_id)
            .unwrap()
            .revoke_guess();
    }
}

impl UserPermissionsRepo for HashMapRoomsStorage {
    async fn mute(&self, room_id: &str, target_user_public_id: &str) {
        self.storage
            .write()
            .await
            .get_mut(room_id)
            .unwrap()
            .users
            .iter_mut()
            .find(|user| user.public_id == *target_user_public_id)
            .unwrap()
            .mute();
    }

    async fn unmute(&self, room_id: &str, target_user_public_id: &str) {
        self.storage
            .write()
            .await
            .get_mut(room_id)
            .unwrap()
            .users
            .iter_mut()
            .find(|user| user.public_id == *target_user_public_id)
            .unwrap()
            .unmute();
    }

    async fn ban(&self, room_id: &str, target_user_public_id: &str) {
        self.storage
            .write()
            .await
            .get_mut(room_id)
            .unwrap()
            .ban_user(target_user_public_id)
    }

    async fn is_muted(&self, room_id: &str, public_user_id: &str) -> bool {
        self.storage
            .read()
            .await
            .get(room_id)
            .unwrap()
            .users
            .iter()
            .find(|user| user.public_id == public_user_id)
            .map_or(false, |user| user.is_muted)
    }

    async fn is_banned(&self, room_id: &str, public_user_id: &str) -> bool {
        self.storage
            .read()
            .await
            .get(room_id)
            .unwrap()
            .banned_public_users_ids
            .iter()
            .any(|public_id| public_id.as_str() == public_user_id)
    }
}

impl RoomInfoRepo for HashMapRoomsStorage {
    async fn status(&self, room_id: &str) -> RoomStatus {
        self.storage.read().await.get(room_id).unwrap().status
    }

    async fn users(&self, room_id: &str) -> Vec<User> {
        self.storage.read().await.get(room_id).unwrap().users()
    }

    async fn messages(&self, room_id: &str) -> Vec<ChatMessage> {
        self.storage
            .read()
            .await
            .get(room_id)
            .unwrap()
            .last_messages
            .iter()
            .cloned()
            .collect()
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
