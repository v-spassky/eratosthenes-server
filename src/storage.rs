use crate::message_types::BriefUserInfoPayload;
use crate::models::{LatLng, Room, RoomStatus, User};
use rand::{distributions::Alphanumeric, Rng};
use std::collections::HashMap;
use std::sync::{atomic::AtomicUsize, Arc};
use tokio::sync::{mpsc, RwLock};
use warp::ws::Message;

pub static NEXT_USER_ID: AtomicUsize = AtomicUsize::new(1);

pub type ClientSockets = Arc<RwLock<HashMap<usize, mpsc::UnboundedSender<Message>>>>;

#[derive(Clone, Default)]
pub struct Rooms {
    storage: Arc<RwLock<HashMap<String, Room>>>,
}

impl Rooms {
    pub async fn such_room_exists(&self, room_id: &str) -> bool {
        self.storage.read().await.contains_key(room_id)
    }

    pub async fn room_has_such_user(&self, room_id: &str, user_id: &str) -> bool {
        self.storage
            .read()
            .await
            .get(room_id)
            .unwrap()
            .users
            .iter()
            .any(|user| user.name == user_id)
    }

    pub async fn user_is_host_of_the_room(&self, room_id: &str, user_id: &str) -> bool {
        self.storage
            .read()
            .await
            .get(room_id)
            .unwrap()
            .users
            .iter()
            .find(|user| user.name == user_id)
            .map_or(false, |user| user.is_host)
    }

    pub async fn users_of_room_as_json(&self, room_id: &str) -> String {
        self.storage
            .read()
            .await
            .get(room_id)
            .unwrap()
            .users_as_json()
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

    pub async fn submit_user_guess(&self, room_id: &str, user_id: &str, guess: LatLng) {
        self.storage
            .write()
            .await
            .get_mut(room_id)
            .unwrap()
            .users
            .iter_mut()
            .find(|user| user.name == *user_id)
            .unwrap()
            .submit_guess(guess);
    }

    pub async fn create_room(&self) -> String {
        let room_id = generate_room_id();
        let room = Room {
            users: vec![],
            messages: vec![],
            status: RoomStatus::Waiting {
                previous_location: None,
            },
        };
        self.storage.write().await.insert(room_id.clone(), room);
        room_id
    }

    pub async fn handle_new_user_connected(
        &self,
        room_id: &str,
        msg_payload: BriefUserInfoPayload,
        socket_id: usize,
    ) -> Result<(), ()> {
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
            .any(|user| user.name == msg_payload.username);
        if such_user_already_in_the_room {
            return Err(());
        }
        storage_guard
            .get_mut(room_id)
            .unwrap()
            .users
            .push(User::new(
                msg_payload.username,
                msg_payload.avatar_emoji,
                room_has_no_members,
                description_ids_of_room_members,
                socket_id,
            ));
        Ok(())
    }

    pub async fn handle_user_reconnected(
        &self,
        room_id: &str,
        msg_payload: BriefUserInfoPayload,
        socket_id: usize,
    ) {
        self.storage
            .write()
            .await
            .get_mut(room_id)
            .unwrap()
            .users
            .iter_mut()
            .find(|user| user.name == msg_payload.username)
            .unwrap()
            .socket_id = Some(socket_id);
    }

    pub async fn handle_user_disconnected(&self, room_id: &str, msg_payload: BriefUserInfoPayload) {
        let mut storage_guard = self.storage.write().await;
        let user_is_host = storage_guard
            .get(room_id)
            .unwrap()
            .users
            .iter()
            .find(|user| user.name == msg_payload.username)
            .unwrap()
            .is_host;
        storage_guard
            .get_mut(room_id)
            .unwrap()
            .users
            .retain(|user| user.name != msg_payload.username);
        if user_is_host {
            storage_guard.get_mut(room_id).unwrap().reassign_host()
        }
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

    pub async fn handle_game_started(&self, room_id: &str) {
        self.storage
            .write()
            .await
            .get_mut(room_id)
            .unwrap()
            .start_playing();
    }

    pub async fn handle_game_finished(&self, room_id: &str) {
        self.storage
            .write()
            .await
            .get_mut(room_id)
            .unwrap()
            .finish_game();
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
}

fn generate_room_id() -> String {
    rand::thread_rng()
        .sample_iter(&Alphanumeric)
        .take(10)
        .map(char::from)
        .collect()
}
