use crate::map_locations::{get_guess_score, get_random_position};
use crate::user_descriptions::get_random_user_description;

#[derive(Clone, Debug)]
pub struct Room {
    pub users: Vec<User>,
    pub messages: Vec<ChatMessage>,
    pub status: RoomStatus,
}

impl Room {
    pub fn reassign_host(&mut self) {
        if self.users.is_empty() {
            return;
        }
        self.users[0].is_host = true;
    }

    pub fn start_playing(&mut self) {
        self.status = RoomStatus::Playing {
            current_location: get_random_position(),
        };

        for user in self.users.iter_mut() {
            user.last_guess = None;
        }
    }

    pub fn finish_game(&mut self) {
        let prev_position = match &self.status {
            RoomStatus::Playing { current_location } => *current_location,
            _ => {
                eprintln!("Tried to finish game when not playing");
                return;
            }
        };
        self.status = RoomStatus::Waiting {
            previous_location: Some(prev_position),
        };

        for user in self.users.iter_mut() {
            if let Some(guess) = user.last_guess {
                user.score += get_guess_score(guess, prev_position);
            }
        }
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

#[derive(Clone, Debug)]
pub enum RoomStatus {
    Waiting { previous_location: Option<LatLng> },
    Playing { current_location: LatLng },
}

impl RoomStatus {
    pub fn as_json(&self) -> String {
        match self {
            RoomStatus::Waiting { previous_location } => {
                match previous_location {
                    Some(location) => format!("{{\"type\": \"waiting\", \"previousLocation\": {}}}", location.as_json()),
                    None => "{\"type\": \"waiting\", \"previousLocation\": null}".to_string(),
                }
            }
            RoomStatus::Playing { current_location } => {
                format!("{{\"type\": \"playing\", \"currentLocation\": {}}}", current_location.as_json())
            }
        }
    }
}

#[derive(Clone, Debug)]
pub struct User {
    pub name: String,
    pub avatar_emoji: String,
    pub score: u64,
    pub is_host: bool,
    pub description: String,
    pub description_id: usize,
    pub socket_id: usize,
    pub last_guess: Option<LatLng>,
}

impl User {
    pub fn new(
        name: String,
        avatar_emoji: String,
        room_has_no_members: bool,
        desc_exclusion_list: Vec<usize>,
        socket_id: usize,
    ) -> Self {
        let (description_id, description) = get_random_user_description(desc_exclusion_list);
        User {
            name,
            avatar_emoji,
            score: 0,
            is_host: room_has_no_members,
            description,
            description_id,
            socket_id,
            last_guess: None,
        }
    }

    pub fn submit_guess(&mut self, guess: LatLng) {
        self.last_guess = Some(guess);
    }

    pub fn as_json(&self) -> String {
        format!(
            "{{\"name\": \"{}\", \"avatarEmoji\": \"{}\", \"isHost\": {},\"score\": {},
            \"description\": \"{}\", \"lastGuess\": {}}}",
            self.name,
            self.avatar_emoji,
            self.is_host,
            self.score,
            self.description,
            self.last_guess_as_json(),
        )
    }

    fn last_guess_as_json(&self) -> String {
        match &self.last_guess {
            Some(guess) => guess.as_json(),
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
    pub author_name: String,
    pub content: String,
}
