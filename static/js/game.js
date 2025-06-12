let eventSource;
let gameCode = null;
let playerId = null;
let authToken = null;

document.addEventListener('DOMContentLoaded', () => {
    const serverContextElement = document.getElementById('server-context');
    if (!serverContextElement) {
        // We are on a page without context, like the welcome page.
        return;
    }

    const serverContext = JSON.parse(serverContextElement.textContent);

    // If we are on a game page, initialize the game state.
    if (serverContext.game_code && serverContext.auth_token && serverContext.player_id) {
        gameCode = serverContext.game_code;
        playerId = serverContext.player_id;
        authToken = serverContext.auth_token;

        connectToGameStream();
        showScreen('gameLobby'); // Initially show lobby
    }

    // Add event listeners for game controls
    const closeButton = document.querySelector('.title-bar-controls button[aria-label="Close"]');
    if(closeButton) closeButton.addEventListener('click', leaveGame);

    const leaveButton = document.querySelector('#lobbyActions button');
    if(leaveButton) leaveButton.addEventListener('click', leaveGame);

    const startGameButton = document.getElementById('startGameBtn');
    if(startGameButton) startGameButton.addEventListener('click', startGame);

    const assassinateButton = document.querySelector('#gamePlaying button');
    if(assassinateButton) assassinateButton.addEventListener('click', eliminateTarget);

    const backToMenuButtonFinished = document.querySelector('#gameFinished button');
    if(backToMenuButtonFinished) backToMenuButtonFinished.addEventListener('click', leaveGame);
    
    const backToMenuButtonDeath = document.querySelector('#deathScreen button');
    if(backToMenuButtonDeath) backToMenuButtonDeath.addEventListener('click', leaveGame);
});


function showScreen(screenId) {
    document.querySelectorAll('#gameView > .screen').forEach(screen => {
        screen.style.display = 'none';
    });
    const screen = document.getElementById(screenId);
    if (screen) {
        screen.style.display = 'block';
    }
}

function showGameView() {
    document.getElementById('welcomeView').style.display = 'none';
    document.getElementById('gameView').style.display = 'block';
}

function connectToGameStream() {
    if (eventSource) {
        eventSource.close();
    }

    let url = `${API_BASE_URL}/events/${gameCode}`;
    if (playerId) {
        url += `?player_id=${playerId}`;
    }

    eventSource = new EventSource(url);
    eventSource.onmessage = (event) => {
        const { game, players } = JSON.parse(event.data);
        updateGameState(game, players);
    };
    eventSource.onerror = (err) => {
        console.error('EventSource failed:', err);
        eventSource.close();
        // Don't auto-leave, maybe the connection can be re-established or the user wants to see the last state.
        alert('Connection to the game has been lost.');
    };
}


function updateGameState(game, players) {
    const me = players.find(p => p.id === playerId);
    
    if (!me) {
        alert("You have been removed from the game.");
        leaveGame();
        return;
    }

    const gameStatus = game.status.toLowerCase();

    if (gameStatus === 'lobby') {
        updateLobby(game, players);
        showScreen('gameLobby');
    } else if (gameStatus === 'inprogress') {
        if (me.is_alive) {
            updateGameScreen(game, players, me);
            showScreen('gamePlaying');
        } else {
            const killer = players.find(p => p.id === me.killed_by);
            updateDeathScreen(killer);
            showScreen('deathScreen');
        }
    } else if (gameStatus === 'finished') {
        const winner = players.find(p => p.is_alive);
        updateFinishedScreen(winner);
        showScreen('gameFinished');
    }
}

function updateLobby(game, players) {
    document.getElementById('lobbyGameName').textContent = `Game Lobby: ${game.code}`;

    // Ensure lobby-specific sections are visible
    document.getElementById('rejoinLinkContainer').style.display = 'block';
    document.getElementById('inviteContainer').style.display = 'block';
    document.getElementById('playerListContainer').style.display = 'block';
    document.getElementById('lobbyActions').style.display = 'flex'; // Use flex for field-row

    const shareLink = `${window.location.origin}/game/${game.code}`;
    const shareLinkInput = document.getElementById('shareLink');
    shareLinkInput.value = shareLink;
    document.getElementById('copyLinkBtn').onclick = () => copyToClipboard(shareLink, 'Game link copied to clipboard!');

    const rejoinLink = `${window.location.origin}/game/${gameCode}/player/${authToken}`;
    const rejoinLinkInput = document.getElementById('rejoinLink');
    rejoinLinkInput.value = rejoinLink;
    document.getElementById('copyRejoinLinkBtn').onclick = () => copyToClipboard(rejoinLink, 'Rejoin link copied to clipboard!');

    const qrCodeImg = document.getElementById('qrCode');
    if (qrCodeImg) {
        qrCodeImg.src = `https://api.qrserver.com/v1/create-qr-code/?size=150x150&data=${encodeURIComponent(shareLink)}`;
    }
    
    const playerList = document.getElementById('playerList');
    playerList.innerHTML = '';
    players.forEach(p => {
        const li = document.createElement('li');
        li.textContent = `${p.name} ${p.id === game.host_id ? '(Host)' : ''}`;
        if (p.id === playerId) {
            li.style.fontWeight = 'bold';
        }
        playerList.appendChild(li);
    });

    const me = players.find(p => p.id === playerId);
    const startGameBtn = document.getElementById('startGameBtn');
    if (me && me.id === game.host_id) {
        startGameBtn.style.display = 'block';
    } else {
        startGameBtn.style.display = 'none';
    }
}

function updateGameScreen(game, players, me) {
    document.getElementById('gameViewTitle').textContent = "Game in Progress";

    // Hide lobby-specific sections
    document.getElementById('rejoinLinkContainer').style.display = 'none';
    document.getElementById('inviteContainer').style.display = 'none';
    document.getElementById('playerListContainer').style.display = 'none';
    document.getElementById('lobbyActions').style.display = 'none';
    document.getElementById('lobbyGameName').style.display = 'none';

    const targetInfo = document.getElementById('targetInfo');
    if (me.target_name) {
        targetInfo.innerHTML = `<legend>Your Target</legend><p>Your target is: <strong>${me.target_name}</strong></p>`;
    } else {
         targetInfo.innerHTML = `<legend>Your Target</legend><p>Waiting for target...</p>`;
    }

    const gamePlayerList = document.getElementById('gamePlayerList');
    gamePlayerList.innerHTML = '';
    players.filter(p => p.is_alive).forEach(p => {
        const item = document.createElement('li');
        let text = p.name;
        if (p.id === playerId) {
            item.style.fontWeight = 'bold';
            text += ' (You)';
        }
        item.textContent = text;
        gamePlayerList.appendChild(item);
    });
}

function updateDeathScreen(killer) {
    document.getElementById('gameViewTitle').textContent = "You've Been Eliminated";
    const killerInfo = document.getElementById('killerInfo');
    if (killer) {
        killerInfo.innerHTML = `<legend>Eliminated</legend><p>You were eliminated by: <strong>${killer.name}</strong></p>`;
    } else {
        killerInfo.innerHTML = `<legend>Eliminated</legend><p>You were eliminated.</p>`;
    }
}


function updateFinishedScreen(winner) {
    document.getElementById('gameViewTitle').textContent = "Game Over!";
    const winnerInfo = document.getElementById('winnerInfo');
    if (winner) {
        winnerInfo.innerHTML = `<legend>Winner!</legend><p><strong>${winner.name}</strong> has won the game!</p>`;
    } else {
        winnerInfo.innerHTML = `<legend>Winner!</legend><p>The game has finished!</p>`;
    }
}

async function startGame() {
    try {
        const response = await fetch(`${API_BASE_URL}/api/game/${gameCode}/start`, {
            method: 'POST',
            headers: {
                'Content-Type': 'application/json',
                'Authorization': `Bearer ${authToken}`
            },
            body: JSON.stringify({ auth_token: authToken })
        });
        if (!response.ok) {
            let errorText = await response.text();
            try {
                // Try to parse as JSON, as the server might send a JSON error response
                const errorJson = JSON.parse(errorText);
                throw new Error(errorJson.message || 'Failed to start game');
            } catch (e) {
                // If parsing fails, the response was not JSON. Use the raw text.
                // This will fix the "Unexpected token 'E'" error.
                throw new Error(errorText || 'Failed to start game');
            }
        }
        // Game state will update via SSE
    } catch (error) {
        console.error('Error starting game:', error);
        alert(`Could not start game: ${error.message}`);
    }
}

async function eliminateTarget() {
    try {
        const response = await fetch(`${API_BASE_URL}/api/game/${gameCode}/eliminate`, {
            method: 'POST',
            headers: {
                'Authorization': `Bearer ${authToken}`
            }
        });

        if (!response.ok) {
            const error = await response.json();
            throw new Error(error.message || 'Failed to eliminate target');
        }
        // Game state will update via SSE
    } catch (error) {
        console.error('Error eliminating target:', error);
        alert(`Could not eliminate target: ${error.message}`);
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
    }, (err) => {
        console.error('Could not copy text: ', err);
        alert('Failed to copy.');
    });
} 