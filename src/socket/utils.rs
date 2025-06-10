use rand::prelude::*;

pub fn generate_game_code(len: usize) -> String {
    const CHARSET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZ1234567890";
    let mut rng = rand::rng();

    CHARSET
        .choose_multiple(&mut rng, len)
        .map(|&c| c as char)
        .collect()
} 