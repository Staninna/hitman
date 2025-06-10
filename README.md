# Project Hitman: Live Action Assassin Game Backend

This repository contains the Rust backend for a real-time, live-action "Hitman" game. The server uses a hybrid architecture: **Socket.IO** for live lobby and game state updates, and a **RESTful HTTP endpoint** for handling the core "kill" action. The **SQLite database** is the single source of truth for all game state, ensuring data persistence and consistency.

## Core Gameplay

Hitman is a game for two or more players in a real-world setting.

1.  **The Ring:** At the start of the game, every player is assigned another player as their "target," forming a closed loop.
2.  **The QR Code:** Each player's app displays a unique QR code. This code contains a secret URL that identifies them as a target.
3.  **The Kill:** To eliminate a target, a player must use their in-game camera to scan their target's QR code. This action hits a secure server endpoint. The kill is only valid if the target is not in the immediate vicinity of any other active player (this rule is enforced by the players themselves).
4.  **The Inheritance:** When a player successfully eliminates their target, they inherit the target of the person they just eliminated.
5.  **The Winner:** The game continues until only one player remains.

## Tech Stack

*   **Language:** [**Rust**](https://www.rust-lang.org/)
*   **Web Framework:** [**Axum**](https://github.com/tokio-rs/axum) - A modern and ergonomic web framework for building the REST API and serving the Socket.IO layer.
*   **Real-time Communication:** [**Socketioxide**](https://github.com/Totodore/socketioxide) - A powerful Socket.IO server implementation that integrates directly into Axum as a service layer.
*   **Database:** [**SQLite**](https://www.sqlite.org/index.html) - A lightweight, file-based SQL database for persisting all game and player data.
*   **Database Toolkit:** [**SQLx**](https://github.com/launchbadge/sqlx) - A modern, async-ready SQL toolkit for Rust that provides compile-time query checking.
*   **Async Runtime:** [**Tokio**](https://tokio.rs/)
*   **Serialization:** [**Serde**](https://serde.rs/)
*   **Utilities:**
    *   `uuid`: For generating unique player secrets.
    *   `rand`: For generating unique game codes and shuffling players.
    *   `tracing`: For structured logging.

## Architecture

The server is designed for robustness and simplicity, with the database as the central component.

1.  **Database as the Single Source of Truth:** All games, players, and their states are stored in a SQLite database. There is no in-memory caching of game state; every request reads directly from the database. This guarantees that all players see the most up-to-date information and makes the server resilient to restarts.

2.  **Shared Database Pool:** An `sqlx::SqlitePool` connection pool is created when the server starts and is shared across all handlers as Axum application state. This allows for efficient, concurrent database access.

3.  **Hybrid API:**
    *   **Socket.IO:** Used for low-latency, real-time events like creating a lobby, players joining, starting the game, and broadcasting state changes (like eliminations) to all clients.
    *   **REST API (Axum):** A secure HTTP endpoint (`POST /api/kill`) is used for the critical "kill" action. This is more robust for a single, important event triggered by a QR scan.

4.  **Atomic Operations:** All state-mutating operations (joining a game, starting a game, processing a kill) are performed within a **database transaction**. This ensures that the operation either completes fully or not at all, preventing the database from ever entering an inconsistent state.

## Data Models & Database Schema

### Rust Structs

These structs are direct representations of the data stored in the database tables.

```rust
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Serialize, Deserialize, Debug, Clone, sqlx::Type)]
#[sqlx(type_name = "game_status", rename_all = "lowercase")]
pub enum GameStatus {
    Lobby,
    InProgress,
    Finished,
}

#[derive(Serialize, Deserialize, Debug, Clone, sqlx::FromRow)]
pub struct Player {
    pub id: i64,
    pub name: String,
    #[serde(skip_serializing)] // Never send secrets or tokens to all clients
    pub secret_code: Uuid,
    #[serde(skip_serializing)]
    pub auth_token: String,
    pub is_alive: bool,
    pub target_id: Option<i64>,
    pub game_id: i64,
}

#[derive(Debug, Clone, sqlx::FromRow)]
pub struct Game {
    pub id: i64,
    pub code: String,
    pub status: GameStatus,
    pub host_id: i64,
    pub winner_id: Option<i64>,
}
```

### Database Schema (SQLite)

```sql
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
    FOREIGN KEY (target_id) REFERENCES players(id) ON DELETE SET NULL
);
```

## API Definitions

### Socket.IO Events API

Handles real-time lobby and game state synchronization.

**Client-to-Server Events**

| Event Name | Payload | Description |
| :--- | :--- | :--- |
| `create_game` | `{"player_name": "string"}` | Requests to create a new game. |
| `join_game` | `{"game_code": "string", "player_name": "string"}` | Requests to join an existing game lobby. |
| `start_game` | `{"game_code": "string"}` | The host requests to start the game. |

**Server-to-Client Events**

| Event Name | Payload | Description |
| :--- | :--- | :--- |
| `game_created` | `{"game_code": "string", "player_secret": "uuid", "auth_token": "string", "players": [Player]}` | Sent *only* to the creator with their private data and the game state. |
| `join_success` | `{"game_code": "string", "player_secret": "uuid", "auth_token": "string", "players": [Player]}` | Sent *only* to the joining player with their private data and the game state. |
| `player_joined` | `{"players": [Player]}` | Broadcast to all other players in a game room when a new player joins. |
| `game_started` | `{"players": [Player]}` | Broadcast to all players when the game begins. Each client app uses this to find and display the player's target. |
| `player_eliminated` | `{"eliminated_player_name": "string", "killer_name": "string"}` | Broadcast to all players when a kill is confirmed. |
| `new_target` | `{"target_name": "string"}` | Sent to a specific player after a successful kill, informing them of their new target. |
| `game_over` | `{"winner_name": "string"}` | Broadcast to all players when a winner is determined. |
| `error` | `{"message": "string"}` | Sent to a specific client when an action fails. |

### REST API Endpoint

Handles the high-stakes "kill" action.

*   **Endpoint:** `POST /api/kill`
*   **Description:** A player (the killer) submits their target's secret to confirm an elimination.
*   **Headers:**
    *   `Authorization: Bearer <killer_auth_token>`
*   **Query Parameters:**
    *   `secret=<target_secret_code>` (The UUID from the target's QR code)
*   **Success Response (200 OK):**
    *   Body: `{"message": "Kill confirmed"}`
*   **Error Responses:**
    *   `401 Unauthorized`: Killer's auth token is missing or invalid.
    *   `403 Forbidden`: The identified target is not the killer's current target.
    *   `404 Not Found`: The target secret does not correspond to an active player.
    *   `422 Unprocessable Entity`: The game is not in progress.

## Game Flow Walkthrough

1.  **Game Creation:**
    *   Alice sends a `create_game` event.
    *   **Server:**
        1.  Starts a database transaction.
        2.  Inserts a new `Game` into the `games` table.
        3.  Inserts a new `Player` (Alice) into the `players` table, generating her `secret_code` and `auth_token`.
        4.  Updates the new game's `host_id` to point to Alice's new player ID.
        5.  Commits the transaction.
        6.  Responds to Alice's socket with the `game_created` event, containing the game code and her private credentials.

2.  **The Kill:**
    *   Alice's target is Bob. Bob's app displays a QR code containing the URL `https://hitman.game/kill?secret=BOB_SECRET_UUID`.
    *   Alice scans the code and her app makes a `POST` request to `/api/kill`.
    *   **Server:**
        1.  Starts a database transaction.
        2.  Queries the `players` table to find the player (Alice) whose `auth_token` matches the one in the `Authorization` header.
        3.  Queries the `players` table to find the player (Bob) whose `secret_code` matches the one in the query parameter.
        4.  **Validates:** Is Alice's `target_id` equal to Bob's `id`? Is the game `InProgress`?
        5.  **Updates:** Sets Bob's `is_alive` to `false`. Updates Alice's `target_id` to Bob's old `target_id`.
        6.  Commits the transaction.
        7.  After the transaction succeeds, it uses `socketioxide` to broadcast the `player_eliminated` event and sends the private `new_target` event to Alice.

## Getting Started

1.  **Prerequisites:**
    *   Install the Rust toolchain: `curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh`
2.  **Clone & Setup:**
    ```bash
    git clone <repository-url>
    cd project-hitman-backend
    # Create the environment file. The server will create and migrate the database on first run.
    cp .env.example .env # Edit DATABASE_URL if needed
    ```
3.  **Build and Run:**
    ```