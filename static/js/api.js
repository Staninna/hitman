import { gameState } from './state.js';
import { updateGameState } from './uimanager.js';

const API_BASE_URL = `${window.location.protocol}//${window.location.host}`;

async function fetchApi(url, options = {}) {
    const { authToken } = gameState.getGameDetails();
    const headers = {
        'Authorization': `Bearer ${authToken}`,
        'Content-Type': 'application/json',
        ...options.headers,
    };

    const response = await fetch(url, { ...options, headers });

    if (!response.ok) {
        throw new Error(`API request failed with status ${response.status}`);
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
            alert("The game session could not be found.");
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