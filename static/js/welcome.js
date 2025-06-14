// This file is self-contained and does not share code with the main game logic.

// --- UI Helpers ---

function showToast(message, type = 'info', duration = 3000) {
    const toast = document.createElement('div');
    const toastId = `toast-${Date.now()}`;
    toast.id = toastId;
    toast.className = `toast toast-${type}`;
    toast.textContent = message;

    // Basic styling
    toast.style.position = 'fixed';
    toast.style.top = '20px';
    toast.style.right = '20px';
    toast.style.padding = '15px';
    toast.style.backgroundColor = type === 'error' ? '#c00' : '#007b0c';
    toast.style.color = 'white';
    toast.style.borderRadius = '5px';
    toast.style.zIndex = '1000';
    toast.style.opacity = '0';
    toast.style.transition = 'opacity 0.5s ease-in-out';

    document.body.appendChild(toast);

    // Fade in
    setTimeout(() => {
        toast.style.opacity = '1';
    }, 100);

    // Fade out
    setTimeout(() => {
        toast.style.opacity = '0';
        setTimeout(() => {
            document.body.removeChild(toast);
        }, 500); // Wait for transition to finish
    }, duration);
}

function showModal(modalId) {
    const modal = document.getElementById(modalId);
    if (modal) {
        modal.style.display = 'flex';
    }
}

function hideModal(modalId) {
    const modal = document.getElementById(modalId);
    if (modal) {
        modal.style.display = 'none';
    }
}

// --- API Helpers ---

class ApiError extends Error {
    constructor(message, status) {
        super(message);
        this.name = 'ApiError';
        this.status = status;
    }
}

async function fetchApi(url, options = {}) {
    const headers = {
        'Content-Type': 'application/json',
        ...options.headers,
    };

    const response = await fetch(url, { ...options, headers });

    if (!response.ok) {
        let message = `API request failed with status ${response.status}`;
        try {
            const data = await response.clone().json();
            if (data && data.error) {
                message = data.error;
            }
        } catch (_) {
            // Ignore and fall back to default message
        }
        throw new ApiError(message, response.status);
    }

    if (response.status === 204) {
        return null;
    }
    return response.json();
}

const createGame = (playerName) =>
	fetchApi("/api/game", {
		method: "POST",
		body: JSON.stringify({ player_name: playerName }),
	});

const joinGame = (gameCode, playerName) =>
	fetchApi(`/api/game/${gameCode}/join`, {
		method: "POST",
		body: JSON.stringify({ player_name: playerName }),
	});

// --- Main Logic ---

document.addEventListener('DOMContentLoaded', () => {
    const serverContextElement = document.getElementById('server-context');
    let serverContext = {};
    if (serverContextElement && serverContextElement.textContent) {
        try {
            serverContext = JSON.parse(serverContextElement.textContent);
        } catch (e) {
            console.error("Could not parse server context:", e);
        }
    }

    document.getElementById('createGameBtn')?.addEventListener('click', () => showModal('createGameModal'));
    document.getElementById('joinGameBtn')?.addEventListener('click', () => showModal('joinGameModal'));

    document.getElementById('createGameModalClose')?.addEventListener('click', () => hideModal('createGameModal'));
    document.getElementById('createGameCancel')?.addEventListener('click', () => hideModal('createGameModal'));

    document.getElementById('joinGameModalClose')?.addEventListener('click', () => hideModal('joinGameModal'));
    document.getElementById('joinGameCancel')?.addEventListener('click', () => hideModal('joinGameModal'));

    if (serverContext.is_game_page && serverContext.game_exists) {
        const gameIdInput = document.getElementById('gameId');
        if (gameIdInput) {
            gameIdInput.value = serverContext.game_code;
        }
        showModal('joinGameModal');
    }

    document.getElementById('createGameConfirm')?.addEventListener('click', async () => {
        const creatorNameInput = document.getElementById('creatorName');
        const creatorName = creatorNameInput ? creatorNameInput.value : '';
        if (!creatorName) {
            showToast('Please enter your name.', 'error');
            return;
        }

        try {
            const data = await createGame(creatorName);
            window.location.href = `/game/${data.game_code}/player/${data.auth_token}/lobby`;
        } catch (error) {
            showToast(error.message, 'error');
        }
    });

    document.getElementById('joinGameConfirm')?.addEventListener('click', async () => {
        const gameIdInput = document.getElementById('gameId');
        const playerNameInput = document.getElementById('playerName');
        
        const gameId = gameIdInput ? gameIdInput.value : '';
        const playerName = playerNameInput ? playerNameInput.value : '';

        if (!gameId || !playerName) {
            showToast('Please enter both game ID and your name.', 'error');
            return;
        }

        try {
            const data = await joinGame(gameId, playerName);
            window.location.href = `/game/${gameId}/player/${data.auth_token}/lobby`;
        } catch (error) {
            showToast(error.message, 'error');
        }
    });
}); 