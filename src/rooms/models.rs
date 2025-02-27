use crate::map::{self, models::LatLng};
use crate::rooms::consts::ROUNDS_PER_GAME;
use crate::storage::consts::HOW_MUCH_LAST_MESSAGES_TO_STORE;
use crate::users::models::User;
use serde::Serialize;
use serde_unit_struct::{Deserialize_unit_struct, Serialize_unit_struct};
use std::collections::VecDeque;
use std::sync::atomic::{AtomicUsize, Ordering};

use super::message_types::BotMessagePayload;

pub static NEXT_CHAT_MESSAGE_ID: AtomicUsize = AtomicUsize::new(1);

#[derive(Clone, Debug)]
pub struct Room {
    pub users: Vec<User>,
    pub last_messages: VecDeque<ChatMessage>,
    pub status: RoomStatus,
    pub banned_public_users_ids: Vec<String>,
    pub rounds_left: u64,
}

impl Room {
    pub fn reassign_host(&mut self) {
        if self.users.is_empty() {
            return;
        }
        self.users[0].is_host = true;
    }

    pub fn start_playing(&mut self) {
        let new_game = self.rounds_left == ROUNDS_PER_GAME;
        self.status = RoomStatus::Playing {
            current_location: map::locations::random(),
        };
        for user in self.users.iter_mut() {
            user.last_guess = None;
            if new_game {
                user.score = 0;
            }
        }
    }

    pub fn finish_game(&mut self) -> bool {
        let prev_position = match &self.status {
            RoomStatus::Playing { current_location } => *current_location,
            _ => {
                eprintln!("Tried to finish game when not playing");
                return self.rounds_left == 0;
            }
        };
        self.status = RoomStatus::Waiting {
            previous_location: Some(prev_position),
        };
        for user in self.users.iter_mut() {
            if let Some(guess) = user.last_guess {
                let last_round_score = map::estimate_guess(guess, prev_position);
                user.last_round_score = Some(last_round_score);
                user.score += last_round_score;
            } else {
                user.last_round_score = None;
            }
            user.submitted_guess = false;
        }
        self.rounds_left = self.rounds_left.saturating_sub(1);
        let game_finished = self.rounds_left == 0;
        if game_finished {
            self.rounds_left = ROUNDS_PER_GAME;
        }
        game_finished
    }

    pub fn add_message(&mut self, message: ChatMessage) {
        if self.last_messages.len() >= HOW_MUCH_LAST_MESSAGES_TO_STORE {
            self.last_messages.pop_front();
        }
        self.last_messages.push_back(message);
    }

    pub fn ban_user(&mut self, target_user_public_id: &str) {
        self.users
            .retain(|user| user.public_id != target_user_public_id);
        self.banned_public_users_ids
            .push(target_user_public_id.to_string());
    }

    pub fn users(&self) -> Vec<User> {
        // TODO: maintain `self.users` sorted on insertion
        let mut users = self.users.clone();
        users.sort_by(|a, b| b.score.cmp(&a.score));
        users
    }
}

#[derive(Copy, Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum RoomStatus {
    Waiting {
        #[serde(rename = "previousLocation")]
        previous_location: Option<LatLng>,
    },
    Playing {
        #[serde(rename = "currentLocation")]
        current_location: LatLng,
    },
}

#[derive(Clone, Debug, Serialize)]
#[serde(untagged)]
pub enum ChatMessage {
    FromPlayer {
        r#type: FromPlayerChatMessage,
        id: usize,
        #[serde(rename = "authorName")]
        author_name: String,
        content: String,
        #[serde(rename = "attachmentIds")]
        attachment_ids: Vec<String>,
    },
    FromBot {
        r#type: FromBotChatMessage,
        id: usize,
        content: BotMessagePayload,
    },
}

#[derive(Clone, Debug, Serialize_unit_struct, Deserialize_unit_struct)]
pub struct FromPlayerChatMessage;

#[derive(Clone, Debug, Serialize_unit_struct, Deserialize_unit_struct)]
pub struct FromBotChatMessage;

impl ChatMessage {
    pub fn from_player(author_name: String, content: String, attachment_ids: Vec<String>) -> Self {
        let id = NEXT_CHAT_MESSAGE_ID.fetch_add(1, Ordering::Relaxed);
        Self::FromPlayer {
            r#type: FromPlayerChatMessage {},
            id,
            author_name,
            content,
            attachment_ids,
        }
    }
    pub fn from_bot(content: BotMessagePayload) -> Self {
        let id = NEXT_CHAT_MESSAGE_ID.fetch_add(1, Ordering::Relaxed);
        Self::FromBot {
            r#type: FromBotChatMessage {},
            id,
            content,
        }
    }

    pub fn id(&self) -> usize {
        match self {
            Self::FromPlayer { id, .. } => *id,
            Self::FromBot { id, .. } => *id,
        }
    }
}
