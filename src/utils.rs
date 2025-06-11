use rand::Rng as _;

pub fn generate_game_code(len: usize) -> String {
    const CHARSET: &[u8] = b"ABCDEFGHIJKLMNPQRSTUVWXYZ123456789";
    let mut rng = rand::rng();
    let code: String = (0..len)
        .map(|_| {
            let idx = rng.random_range(0..CHARSET.len());
            CHARSET[idx] as char
        })
        .collect();
    tracing::debug!("Generated game code: {}", code);
    code
}
