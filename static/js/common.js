const API_BASE_URL = `${window.location.protocol}//${window.location.host}`;

function showModal(modalId) {
    document.getElementById(modalId).style.display = 'flex';
}

function hideModal(modalId, event) {
    if (event && event.target.id !== modalId) {
        return;
    }
    document.getElementById(modalId).style.display = 'none';
}

let gameCode = null;
let playerId = null;
let authToken = null;
let lastUpdate = 0;
let pollingInterval;

async function pollForChanges() {
    if (!gameCode) return;

    try {
        const response = await fetch(
            `${API_BASE_URL}/api/game/${gameCode}/changed?since=${lastUpdate}`
        );
        if (!response.ok) {
            console.error("Failed to poll for changes:", response.status);
            if (response.status === 404) {
                alert("The game session could not be found.");
                leaveGame();
            }
            return;
        }

        const data = await response.json();
        lastUpdate = data.now;

        if (data.changed) {
            fetchGameState();
        }
    } catch (error) {
        console.error("Error polling for changes:", error);
    }
}

async function fetchGameState() {
    try {
        const response = await fetch(`${API_BASE_URL}/api/game/${gameCode}`);
        if (!response.ok) {
            console.error("Failed to fetch game state:", response.status);
            return;
        }
        const { game, players } = await response.json();
        updateGameState(game, players);
    } catch (error) {
        console.error("Error fetching game state:", error);
    }
}

function startPolling() {
    stopPolling();
    // Fetch initial state right away
    fetchGameState();
    // Then start polling for changes
    pollingInterval = setInterval(pollForChanges, 2000); // Poll every 2 seconds
}

function stopPolling() {
    if (pollingInterval) {
        clearInterval(pollingInterval);
        pollingInterval = null;
    }
}

function showScreen(screenId) {
    const screens = document.querySelectorAll('.screen');
    screens.forEach(screen => {
        screen.style.display = screen.id === screenId ? 'block' : 'none';
    });
}

function updateGameState(game, players) {
    const me = players.find(p => p.id === playerId);
    if (!me) {
        alert("You have been removed from the game.");
        leaveGame();
        return;
    }

    const gameStatus = game.status.toLowerCase();
    const currentPath = window.location.pathname;

    const buildPath = (page) => `/game/${game.code}/player/${authToken}/${page}`;

    const handleRedirect = (path) => {
        if (currentPath !== path) {
            window.location.href = path;
        }
    };

    if (gameStatus === 'lobby') {
        const lobbyPath = buildPath('lobby');
        if (currentPath !== lobbyPath) {
            handleRedirect(lobbyPath);
        } else {
            showScreen('gameLobby');
            updateLobby(game, players);
        }
    } else if (gameStatus === 'inprogress') {
        if (me.is_alive) {
            const gamePath = buildPath('game');
            if (currentPath !== gamePath) {
                handleRedirect(gamePath);
            } else {
                showScreen('gamePlaying');
                updateGameScreen(game, players, me);
            }
        } else {
            const eliminatedPath = buildPath('eliminated');
            if (currentPath !== eliminatedPath) {
                handleRedirect(eliminatedPath);
            } else {
                const killer = players.find(p => p.id === me.killed_by);
                showScreen('deathScreen');
                updateDeathScreen(killer);
            }
        }
    } else if (gameStatus === 'finished') {
        const gameOverPath = buildPath('game_over');
        if (currentPath !== gameOverPath) {
            handleRedirect(gameOverPath);
        } else {
            const winner = players.find(p => p.is_alive);
            showScreen('gameFinished');
            updateFinishedScreen(winner);
        }
    }
}

function leaveGame() {
    stopPolling();
    window.location.href = '/';
}

function copyToClipboard(text, successMessage) {
    navigator.clipboard.writeText(text).then(() => {
        alert(successMessage || 'Copied to clipboard!');
    }).catch(err => {
        console.error('Failed to copy text: ', err);
        alert('Failed to copy to clipboard.');
    });
} 