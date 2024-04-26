use rand::distributions::Alphanumeric;
use rand::{thread_rng, Rng};

pub fn generate_user_id() -> String {
    let rng = thread_rng();
    let id_1: String = rng
        .clone()
        .sample_iter(&Alphanumeric)
        .take(3)
        .map(char::from)
        .collect();
    let id_2: String = rng
        .clone()
        .sample_iter(&Alphanumeric)
        .take(3)
        .map(char::from)
        .collect();
    let id_3: String = rng
        .clone()
        .sample_iter(&Alphanumeric)
        .take(3)
        .map(char::from)
        .collect();
    format!("{}e{}R{}", id_1, id_2, id_3)
}

pub fn user_id_is_valid(user_id: &str) -> bool {
    user_id.len() == 11
        && user_id.chars().nth(3).unwrap_or('0') == 'e'
        && user_id.chars().nth(7).unwrap_or('0') == 'R'
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn generated_id_is_valid() {
        let generated_id = generate_user_id();
        assert!(user_id_is_valid(&generated_id));
    }

    #[test]
    fn dummy_id_with_wrong_len_is_not_valid() {
        let dummy_id = "ho ho ho!".to_string();
        assert!(!user_id_is_valid(&dummy_id));
    }

    #[test]
    fn dummy_id_with_right_len_is_not_valid() {
        let dummy_id = "here11chars".to_string();
        assert!(!user_id_is_valid(&dummy_id));
    }
}
