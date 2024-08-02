use crate::map_locations::models::LatLng;
use crate::rooms::models::RoomStatus;
use crate::users::descriptions;
use serde::Serialize;

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct User {
    pub public_id: String,
    #[serde(skip_serializing)]
    pub private_id: String,
    pub name: String,
    pub avatar_emoji: String,
    pub score: u64,
    pub is_host: bool,
    pub description_index: usize,
    #[serde(skip_serializing)]
    pub socket_id: Option<usize>,
    pub last_guess: Option<LatLng>,
    pub submitted_guess: bool,
    pub last_round_score: Option<u64>,
    pub is_muted: bool,
}

impl User {
    pub fn new(
        public_id: String,
        private_id: String,
        name: String,
        avatar_emoji: String,
        room_has_no_members: bool,
        desc_exclusion_list: Vec<usize>,
        socket_id: usize,
    ) -> Self {
        let description_index = descriptions::random_except_these(desc_exclusion_list);
        User {
            private_id,
            public_id,
            name,
            avatar_emoji,
            score: 0,
            is_host: room_has_no_members,
            description_index,
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
}
