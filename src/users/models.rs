use crate::map_locations::models::LatLng;
use crate::rooms::models::RoomStatus;
use crate::users::descriptions;

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
        let (description_id, description) = descriptions::random_except_these(desc_exclusion_list);
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
        self.last_guess = Some(guess);
        if let RoomStatus::Playing { .. } = room_status {
            self.submitted_guess = true;
        }
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
