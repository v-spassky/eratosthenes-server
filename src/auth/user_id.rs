use rand::distributions::Alphanumeric;
use rand::{self, Rng};

pub fn generate() -> String {
    let rng = rand::thread_rng();
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
