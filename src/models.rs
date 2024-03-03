use crate::user_descriptions::get_random_user_description;

#[derive(Clone, Debug)]
pub struct Room {
    pub users: Vec<User>,
    pub messages: Vec<ChatMessage>,
}

impl Room {
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
pub struct User {
    pub name: String,
    pub avatar_emoji: String,
    pub score: u64,
    pub is_host: bool,
    pub description: String,
    pub description_id: usize,
    pub socket_id: usize,
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
        }
    }

    pub fn as_json(&self) -> String {
        format!(
            "{{\"name\": \"{}\", \"avatarEmoji\": \"{}\", \"isHost\": {},\"score\": {},
            \"description\": \"{}\"}}",
            self.name, self.avatar_emoji, self.is_host, self.score, self.description,
        )
    }
}

#[derive(Clone, Debug)]
pub struct ChatMessage {
    pub author_name: String,
    pub content: String,
}
