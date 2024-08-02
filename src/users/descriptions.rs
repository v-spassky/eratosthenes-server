use crate::users::consts::NUMBER_OF_USER_DESCRIPTIONS;
use rand::Rng;

pub fn random_except_these(exclusion_list: Vec<usize>) -> usize {
    let mut rng = rand::thread_rng();
    if exclusion_list.len() >= NUMBER_OF_USER_DESCRIPTIONS {
        let index = rng.gen_range(0..NUMBER_OF_USER_DESCRIPTIONS);
        return index;
    }
    let allowed_indices: Vec<usize> = (0..NUMBER_OF_USER_DESCRIPTIONS)
        .filter(|index| !exclusion_list.contains(index))
        .collect();
    allowed_indices[rng.gen_range(0..allowed_indices.len())]
}
