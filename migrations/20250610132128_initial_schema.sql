-- Stores the overall game sessions
CREATE TABLE IF NOT EXISTS games (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    code TEXT NOT NULL UNIQUE,
    status TEXT NOT NULL DEFAULT 'lobby', -- 'lobby', 'inprogress', 'finished'
    host_id INTEGER, -- Refers to a player ID, can be NULL initially
    winner_id INTEGER,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

-- Stores player data for each game
CREATE TABLE IF NOT EXISTS players (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    game_id INTEGER NOT NULL,
    name TEXT NOT NULL,
    secret_code TEXT NOT NULL UNIQUE, -- This is the UUID for the QR code
    auth_token TEXT NOT NULL UNIQUE, -- This is the token to auth the kill endpoint
    is_alive BOOLEAN NOT NULL DEFAULT TRUE,
    target_id INTEGER, -- The 'id' of the player they are targeting
    FOREIGN KEY (game_id) REFERENCES games(id) ON DELETE CASCADE,
    FOREIGN KEY (target_id) REFERENCES players(id) ON DELETE SET NULL,
    UNIQUE(game_id, name)
);
