const API_BASE_URL = `${window.location.protocol}//${window.location.host}`;

let eventSource;
let gameCode = null;
let playerId = null;
let authToken = null;

document.addEventListener('DOMContentLoaded', () => {
    // Use server context if available, otherwise fall back to URL parsing
    const serverContext = window.serverContext || {};

    if (serverContext.is_rejoin_page) {
        if (serverContext.game_exists) {
            // Rejoining a valid game
            gameCode = serverContext.game_code;
            playerId = serverContext.player_id;
            authToken = serverContext.auth_token;

            console.log("Rejoining game with context:", serverContext);

            connectToGameStream();
            showGameView();
            // The first SSE event will route to the correct screen (lobby, playing, etc.)
        } else {
            // Rejoin link is invalid, show an error on the welcome screen
            document.getElementById('welcomeView').style.display = 'block';
            const title = document.querySelector("#welcomeView .title-bar-text");
            const body = document.querySelector("#welcomeView .window-body");
            if (title) title.textContent = "Rejoin Failed";
            if (body) {
                body.innerHTML = `
                    <h4>Invalid Rejoin Link</h4>
                    <p class="error-message">Your rejoin link is invalid or the game has ended.</p>
                    <p style="margin-bottom: 20px;">You can create a new game or join a different one.</p>
                    <div class="field-row" style="justify-content: center;">
                        <button onclick="showModal('createGameModal')">Create New Game</button>
                        <button onclick="showModal('joinGameModal')">Join Existing Game</button>
                    </div>
                `;
            }
        }
        return; // Stop further execution
    }
    
    if (serverContext.is_game_page && serverContext.game_code) {
        // Pre-fill game code from server context
        document.getElementById('gameId').value = serverContext.game_code;
        
        // Auto-show join modal only if game exists
        if (serverContext.showJoinModal) {
            showModal('joinGameModal');
        }
    } else {
        // Fallback: parse URL for game code (for backward compatibility)
        const path = window.location.pathname;
        const match = path.match(/^\/game\/([a-zA-Z0-9]+)$/);
        if (match) {
            const code = match[1];
            document.getElementById('gameId').value = code;
            showModal('joinGameModal');
        }
    }

    // Modal listeners (these need to be added after DOM is loaded)
    const createBtn = document.getElementById('createGameBtn');
    const joinBtn = document.getElementById('joinGameBtn');
    
    if (createBtn) createBtn.addEventListener('click', () => showModal('createGameModal'));
    if (joinBtn) joinBtn.addEventListener('click', () => showModal('joinGameModal'));
    
    // Add event listeners for modals if they exist
    const createModal = document.getElementById('createGameModal');
    const joinModal = document.getElementById('joinGameModal');
    
    if (createModal) {
        createModal.addEventListener('click', (e) => hideModal('createGameModal', e));
        const modalContent = createModal.querySelector('.window');
        if (modalContent) modalContent.addEventListener('click', e => e.stopPropagation());
    }
    
    if (joinModal) {
        joinModal.addEventListener('click', (e) => hideModal('joinGameModal', e));
        const modalContent = joinModal.querySelector('.window');
        if (modalContent) modalContent.addEventListener('click', e => e.stopPropagation());
    }

    // Button listeners - these elements may not exist on all pages
    const submitCreateBtn = document.getElementById('submitCreateGame');
    const submitJoinBtn = document.getElementById('submitJoinGame');
    const startBtn = document.getElementById('startGameBtn');
    const eliminateBtn = document.getElementById('submitEliminationBtn');
    const leaveBtn = document.getElementById('leaveGameBtn');
    const backToLobbyBtn = document.getElementById('backToLobbyBtn');
    const playAgainBtn = document.getElementById('playAgainBtn');
    
    if (submitCreateBtn) submitCreateBtn.addEventListener('click', createGame);
    if (submitJoinBtn) submitJoinBtn.addEventListener('click', joinGame);
    if (startBtn) startBtn.addEventListener('click', startGame);
    if (eliminateBtn) eliminateBtn.addEventListener('click', eliminateTarget);
    if (leaveBtn) leaveBtn.addEventListener('click', leaveGame);
    if (backToLobbyBtn) backToLobbyBtn.addEventListener('click', leaveGame);
    if (playAgainBtn) playAgainBtn.addEventListener('click', leaveGame);
});

function showModal(modalId) {
    document.getElementById(modalId).style.display = 'flex';
}

function hideModal(modalId, event) {
    if (event && event.target.id !== modalId) {
        return;
    }
    document.getElementById(modalId).style.display = 'none';
}

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

async function createGame() {
    const creatorName = document.getElementById('creatorName').value;
    if (!creatorName) {
        alert('Please enter your name.');
        return;
    }

    try {
        const response = await fetch(`${API_BASE_URL}/api/game`, {
            method: 'POST',
            headers: { 'Content-Type': 'application/json' },
            body: JSON.stringify({ player_name: creatorName })
        });
        if (!response.ok) {
            const error = await response.json();
            throw new Error(error.message || 'Failed to create game');
        }

        const { game_code, player_id, auth_token, players, game } = await response.json();
        gameCode = game_code;
        playerId = player_id;
        authToken = auth_token;
        
        hideModal('createGameModal');
        history.pushState(null, '', `/game/${gameCode}/player/${authToken}`);
        connectToGameStream();
        updateLobby(game, players);
        showGameView();
        showScreen('gameLobby');

    } catch (error) {
        console.error('Error creating game:', error);
        alert(`Could not create game: ${error.message}`);
    }
}


async function joinGame() {
    const gameId = document.getElementById('gameId').value.trim();
    const playerName = document.getElementById('playerName').value.trim();

    if (!gameId || !playerName) {
        alert('Please enter both a game ID and your name.');
        return;
    }

    try {
        const response = await fetch(`${API_BASE_URL}/api/game/${gameId}/join`, {
            method: 'POST',
            headers: { 'Content-Type': 'application/json' },
            body: JSON.stringify({ player_name: playerName })
        });

        if (response.status === 404) {
            alert('Game not found.');
            return;
        }
        if (response.status === 409) {
            alert('A player with that name already exists in the game.');
            return;
        }
        if (!response.ok) {
            const error = await response.json();
            throw new Error(error.message || 'Failed to join game');
        }

        const { game_code, player_id, auth_token, players, game } = await response.json();
        gameCode = game_code;
        playerId = player_id;
        authToken = auth_token;

        hideModal('joinGameModal');
        history.pushState(null, '', `/game/${gameCode}/player/${authToken}`);
        connectToGameStream();
        updateLobby(game, players);
        showGameView();
        showScreen('gameLobby');

    } catch (error) {
        console.error('Error joining game:', error);
        alert(`Could not join game: ${error.message}`);
    }
}

function connectToGameStream() {
    if (eventSource) {
        eventSource.close();
    }
    eventSource = new EventSource(`${API_BASE_URL}/events/${gameCode}`);
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
    const target = players.find(p => p.id === me.target_id);

    const targetInfo = document.getElementById('targetInfo');
    if (target) {
        targetInfo.innerHTML = `<legend>Your Target</legend><p>Your target is: <strong>${target.name}</strong></p><p>Your secret code is: <strong>${me.secret_code}</strong></p>`;
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
        killerInfo.textContent = `You were eliminated by ${killer.name}.`;
    } else {
        killerInfo.textContent = `You have been eliminated.`;
    }
}

function updateFinishedScreen(winner) {
    document.getElementById('gameViewTitle').textContent = "Game Over!";
     const winnerText = document.getElementById('winnerText');
    if (winner) {
        if (winner.id === playerId) {
            winnerText.textContent = "Congratulations, you are the winner!";
        } else {
            winnerText.textContent = `${winner.name} is the winner!`;
        }
    } else {
        winnerText.textContent = "The game has finished, but there is no winner.";
    }
}

async function startGame() {
    if (!authToken) {
        alert("Authentication token not found. Please rejoin the game.");
        return;
    }
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
            const error = await response.json();
            throw new Error(error.message || 'Failed to start game');
        }
    } catch (error) {
        console.error('Error starting game:', error);
        alert(`Could not start game: ${error.message}`);
    }
}

async function eliminateTarget() {
    if (!authToken) {
        alert("Authentication token not found. Please rejoin the game.");
        return;
    }
    const code = document.getElementById('assassinationCode').value;
    if (!code) {
        alert('Please enter the assassination code from your target.');
        return;
    }

    try {
        const response = await fetch(`${API_BASE_URL}/api/game/${gameCode}/eliminate?secret=${code}`, {
            method: 'POST',
            headers: {
                'Authorization': `Bearer ${authToken}`
            }
        });

        if (!response.ok) {
            const errorData = await response.json().catch(() => ({ message: 'Failed to process elimination.' }));
            throw new Error(errorData.message);
        }

        const result = await response.json();
        document.getElementById('assassinationCode').value = '';
        if (result.game_over) {
             // Game over is handled by the SSE event
        } else {
            alert(`Success! You eliminated ${result.eliminated_player_name}. Your new target is ${result.new_target_name}.`);
        }
    } catch (error) {
        console.error('Error eliminating target:', error);
        alert(`Could not eliminate target: ${error.message}`);
    }
}

function leaveGame() {
    if (eventSource) {
        eventSource.close();
    }

    if (gameCode && authToken) {
        const payload = { auth_token: authToken };
        const blob = new Blob([JSON.stringify(payload)], { type: 'application/json' });
        navigator.sendBeacon(`${API_BASE_URL}/api/game/${gameCode}/leave`, blob);
    }
    
    // Reset state and UI immediately
    gameCode = null;
    playerId = null;
    authToken = null;

    document.getElementById('gameView').style.display = 'none';
    document.getElementById('welcomeView').style.display = 'block';
    history.pushState(null, '', '/');
    window.location.reload();
}

function copyToClipboard(text, successMessage) {
    navigator.clipboard.writeText(text).then(() => {
        alert(successMessage || 'Copied to clipboard!');
    }).catch(err => {
        console.error('Failed to copy text: ', err);
        alert('Failed to copy link.');
    });
}

// Add a handler for the back/forward buttons
window.addEventListener('popstate', (event) => {
    // If we are in a game, leaving the page via back button should take us home.
    if (document.getElementById('gameView').style.display === 'block') {
        leaveGame();
    }
});
