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

let eventSource;
let gameCode = null;
let playerId = null;
let authToken = null;

function connectToGameStream() {
    if (eventSource) {
        eventSource.close();
    }

    let url = `${API_BASE_URL}/events/${gameCode}`;
    if (playerId && authToken) {
        url += `?player_id=${playerId}&auth_token=${authToken}`;
    }

    eventSource = new EventSource(url);
    eventSource.onmessage = (event) => {
        const { game, players } = JSON.parse(event.data);
        updateGameState(game, players);
    };
    eventSource.onerror = (err) => {
        console.error('EventSource failed:', err);
        eventSource.close();
        alert('Connection to the game has been lost.');
    };
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
    if (eventSource) {
        eventSource.close();
    }
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