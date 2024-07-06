use crate::map_locations::{get_guess_score, get_random_position};
use crate::storage::{HOW_MUCH_LAST_MESSAGES_TO_STORE, ROUNDS_PER_GAME};
use crate::user_descriptions::get_random_user_description;
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
        println!("------------------");
        println!("GAME STARTED");
        println!("------------------");
        let new_game = self.rounds_left == ROUNDS_PER_GAME;
        self.status = RoomStatus::Playing {
            current_location: get_random_position(),
        };
        for user in self.users.iter_mut() {
            user.last_guess = None;
            if new_game {
                user.score = 0;
            }
        }
    }

    pub fn finish_game(&mut self) -> bool {
        println!("[finish_game @ start]");
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
            println!("[finish_game.users.{} @ start]", &user.name);
            if let Some(guess) = user.last_guess {
                println!("[finish_game]: user {} has guess: {:?}", &user.name, guess);
                let last_round_score = get_guess_score(guess, prev_position);
                println!(
                    "[finish_game]: user {} got score: {:?}",
                    &user.name, last_round_score
                );
                user.last_round_score = Some(last_round_score);
                println!(
                    "[finish_game]: user {} old score: {:?}",
                    &user.name, user.score
                );
                user.score += last_round_score;
                println!(
                    "[finish_game]: user {} new score: {:?}",
                    &user.name, user.score
                );
            } else {
                println!("[finish_game]: user {} has no guess", &user.name);
                println!(
                    "[finish_game]: user {} old score: {:?}",
                    &user.name, user.score
                );
                user.last_round_score = None;
            }
            user.submitted_guess = false;
            println!("[finish_game.users.{} @ end]", &user.name);
        }
        self.rounds_left = self.rounds_left.saturating_sub(1);
        let game_finished = self.rounds_left == 0;
        if game_finished {
            self.rounds_left = ROUNDS_PER_GAME;
        }
        println!("[finish_game @ end]");
        println!("------------------");
        println!("GAME FINISHED");
        println!("------------------");
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
pub struct User {
    pub id: String,
    pub name: String,
    pub avatar_emoji: String,
    pub score: u64,
    pub is_host: bool,
    pub description: String,
    pub description_id: usize,
    pub socket_id: Option<usize>,
    pub last_guess: Option<LatLng>,
    pub submitted_guess: bool,
    pub last_round_score: Option<u64>,
    pub is_muted: bool,
}

impl User {
    pub fn new(
        id: String,
        name: String,
        avatar_emoji: String,
        room_has_no_members: bool,
        desc_exclusion_list: Vec<usize>,
        socket_id: usize,
    ) -> Self {
        let (description_id, description) = get_random_user_description(desc_exclusion_list);
        User {
            id,
            name,
            avatar_emoji,
            score: 0,
            is_host: room_has_no_members,
            description,
            description_id,
            socket_id: Some(socket_id),
            last_guess: None,
            submitted_guess: false,
            last_round_score: None,
            is_muted: false,
        }
    }

    pub fn submit_guess(&mut self, guess: LatLng, room_status: RoomStatus) {
        println!("[{}.submit_guess @ start]", &self.name);
        self.last_guess = Some(guess);
        if let RoomStatus::Playing { .. } = room_status {
            println!(
                "[{}.submit_guess]: set self.submitted_guess to true",
                &self.name
            );
            self.submitted_guess = true;
        }
        println!("[{}.submit_guess @ end]", &self.name);
    }

    pub fn revoke_guess(&mut self) {
        self.submitted_guess = false;
    }

    pub fn mute(&mut self) {
        self.is_muted = true;
    }

    pub fn unmute(&mut self) {
        self.is_muted = false;
    }

    pub fn change_score(&mut self, amount: i64) {
        if amount >= 0 {
            self.score += amount as u64;
        } else {
            self.score = self.score.saturating_sub(-amount as u64);
        }
    }

    pub fn as_json(&self) -> String {
        format!(
            "{{\"name\": \"{}\", \"avatarEmoji\": \"{}\", \"isHost\": {}, \"score\": {},
            \"description\": \"{}\", \"lastGuess\": {}, \"lastRoundScore\": {},
            \"submittedGuess\": {}, \"isMuted\": {}}}",
            self.name,
            self.avatar_emoji,
            self.is_host,
            self.score,
            self.description,
            self.last_guess_as_json(),
            self.last_round_score_as_json(),
            self.submitted_guess,
            self.is_muted,
        )
    }

    fn last_guess_as_json(&self) -> String {
        match &self.last_guess {
            Some(guess) => guess.as_json(),
            None => "null".to_string(),
        }
    }

    fn last_round_score_as_json(&self) -> String {
        match &self.last_round_score {
            Some(score) => score.to_string(),
            None => "null".to_string(),
        }
    }
}

#[derive(Copy, Clone, Debug)]
pub struct LatLng {
    pub lat: f64,
    pub lng: f64,
}

impl LatLng {
    pub fn as_json(&self) -> String {
        format!("{{\"lat\": {}, \"lng\": {}}}", self.lat, self.lng)
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
