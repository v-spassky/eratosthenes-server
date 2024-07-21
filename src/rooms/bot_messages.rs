#[derive(Debug)]
pub enum BotMessage<'a> {
    RoundStarted {
        round_number: u64,
        rounds_per_game: u64,
    },
    RoundEnded {
        round_number: u64,
        rounds_per_game: u64,
    },
    UserConnected {
        username: &'a str,
    },
    UserDisconnected {
        username: &'a str,
    },
}

impl<'a> BotMessage<'a> {
    pub fn to_human_readable(&self) -> String {
        match self {
            Self::RoundStarted {
                round_number,
                rounds_per_game,
            } => format!("Раунд {round_number}/{rounds_per_game} начался."),
            Self::RoundEnded {
                round_number,
                rounds_per_game,
            } => format!("Раунд {round_number}/{rounds_per_game} закончился."),
            Self::UserConnected { username } => format!("К нам присоединился {username}!"),
            Self::UserDisconnected { username } => format!("{username} отключился."),
        }
    }
}
