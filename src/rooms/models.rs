use crate::map_locations::{self, models::LatLng};
use crate::rooms::consts::ROUNDS_PER_GAME;
use crate::storage::consts::HOW_MUCH_LAST_MESSAGES_TO_STORE;
use crate::users::models::User;
use std::collections::VecDeque;

#[derive(Clone, Debug)]
pub struct Room {
    pub users: Vec<User>,
    pub last_messages: VecDeque<ChatMessage>,
    pub status: RoomStatus,
    pub banned_users_ids: Vec<String>,
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
            current_location: map_locations::random(),
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
                let last_round_score = map_locations::estimate_guess(guess, prev_position);
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

    pub fn ban_user(&mut self, username: &str) {
        let target_user_id = self
            .users
            .iter()
            .find(|user| user.name == username)
            .unwrap()
            .id
            .clone();
        self.users.retain(|user| user.id != target_user_id);
        self.banned_users_ids.push(target_user_id);
    }

    pub fn messages_as_json(&self) -> String {
        let messages_as_json: Vec<String> = self
            .last_messages
            .iter()
            .map(|message| message.as_json())
            .collect();
        format!("[{}]", messages_as_json.join(","))
    }

    pub fn users_as_json(&self) -> String {
        let users_sorted_by_score = {
            let mut users = self.users.clone();
            users.sort_by(|a, b| b.score.cmp(&a.score));
            users
        };
        let users_as_json: Vec<String> = users_sorted_by_score
            .iter()
            .map(|user| user.as_json())
            .collect();
        format!("[{}]", users_as_json.join(","))
    }
}

#[derive(Copy, Clone, Debug)]
pub enum RoomStatus {
    Waiting { previous_location: Option<LatLng> },
    Playing { current_location: LatLng },
}

impl RoomStatus {
    pub fn as_json(&self) -> String {
        match self {
            RoomStatus::Waiting { previous_location } => match previous_location {
                Some(location) => format!(
                    "{{\"type\": \"waiting\", \"previousLocation\": {}}}",
                    location.as_json()
                ),
                None => "{\"type\": \"waiting\", \"previousLocation\": null}".to_string(),
            },
            RoomStatus::Playing { current_location } => {
                format!(
                    "{{\"type\": \"playing\", \"currentLocation\": {}}}",
                    current_location.as_json()
                )
            }
        }
    }
}

#[derive(Clone, Debug)]
pub struct ChatMessage {
    pub is_from_bot: bool,
    /// `None` if `is_from_bot` is `true`.
    pub author_name: Option<String>,
    pub content: String,
}

impl ChatMessage {
    pub fn as_json(&self) -> String {
        let author_name = match self.author_name {
            Some(ref name) => name.clone(),
            None => "null".to_string(),
        };
        format!(
            "{{\"from\": \"{}\", \"content\": \"{}\", \"isFromBot\": {}}}",
            author_name, self.content, self.is_from_bot,
        )
    }
}
