import { gameState } from './state.js';
import { updateGameState } from './uimanager.js';
import { showToast } from './ui.js';

const API_BASE_URL = `${window.location.protocol}//${window.location.host}`;

// A small wrapper around fetch that automatically adds auth headers (if present)
// and converts backend error payloads into thrown Error instances with a readable
// message. Whenever an error response is detected a toast notification is shown
// so that the UI immediately communicates the problem to the user.
export async function fetchApi(url, options = {}) {
    const { authToken } = gameState.getGameDetails();

    const headers = {
        'Content-Type': 'application/json',
        ...options.headers,
    };

    // Only attach the Authorization header if we actually have a token –
    // sending an empty/undefined token could cause the backend to reject the request.
    if (authToken) {
        headers['Authorization'] = `Bearer ${authToken}`;
    }

    const response = await fetch(url, { ...options, headers });

    if (!response.ok) {
        let message = `API request failed with status ${response.status}`;
        // Attempt to read the backend error payload (expected format: { error: "..." })
        try {
            const data = await response.clone().json();
            if (data && data.error) {
                message = data.error;
            }
        } catch (_) {
            // Response either wasn't JSON or parsing failed – ignore and fall back to default message
        }

        // Surface the error to the UI so the player gets immediate feedback.
        showToast(message, 'error');
        throw new Error(message);
    }

    return response.json();
}

export async function pollForChanges() {
    const { gameCode } = gameState.getGameDetails();
    if (!gameCode) return;

    const version = gameState.getVersion();

    try {
        const data = await fetchApi(`${API_BASE_URL}/api/game/${gameCode}/changed?version=${version}`);
        if (typeof data.current_version === 'number') {
            gameState.setVersion(data.current_version);
        }
        if (data.changed) {
            await fetchGameState();
        }
    } catch (error) {
        console.error("Error polling for changes:", error);
        if (error.message && error.message.includes('404')) {
            leaveGame();
        }
    }
}

export async function fetchGameState() {
    const { gameCode } = gameState.getGameDetails();
    try {
        const { game, players, version } = await fetchApi(`${API_BASE_URL}/api/game/${gameCode}`);
        if (typeof version === 'number') {
            gameState.setVersion(version);
        }
        updateGameState(game, players);
    } catch (error) {
        console.error("Error fetching game state:", error);
    }
}

export async function leaveGame() {
    const { gameCode, authToken } = gameState.getGameDetails();
    if (gameCode && authToken) {
        try {
            await fetchApi(`${API_BASE_URL}/api/game/${gameCode}/leave`, {
                method: 'POST',
                body: JSON.stringify({}),
            });
        } catch (error) {
            console.error("Error leaving game:", error);
        }
    }
    window.location.href = '/';
}

export async function startGame() {
    const { gameCode, authToken } = gameState.getGameDetails();
    try {
        await fetchApi(`${API_BASE_URL}/api/game/${gameCode}/start`, {
            method: 'POST',
            body: JSON.stringify({ auth_token: authToken }),
        });
    } catch (error) {
        console.error("Error starting game:", error);
        throw new Error('Failed to start game');
    }
}

export async function eliminateTarget(secretCode) {
    const { gameCode, authToken } = gameState.getGameDetails();
    try {
        await fetchApi(`${API_BASE_URL}/api/game/${gameCode}/eliminate`, {
            method: 'POST',
            body: JSON.stringify({ secret_code: secretCode }),
        });
    } catch (error) {
        console.error("Error eliminating target:", error);
        throw new Error('Failed to eliminate target');
    }
} 