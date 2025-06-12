const API_BASE_URL = `${window.location.protocol}//${window.location.host}`;

let eventSource;
let gameCode = null;
let playerId = null;

document.addEventListener('DOMContentLoaded', () => {
    const path = window.location.pathname;
    const match = path.match(/^\/game\/([a-zA-Z0-9]+)$/);
    if (match) {
        const code = match[1];
        // Pre-fill game code if in URL, then show join modal
        document.getElementById('gameId').value = code;
        showModal('joinGameModal');
    }
});

function showModal(modalId) {
    document.getElementById(modalId).style.display = 'flex';
}

function hideModal(modalId, event) {
    // If event is provided, check if the click was on the overlay itself
    if (event && event.target.id !== modalId) {
        return;
    }
    document.getElementById(modalId).style.display = 'none';
}

function showScreen(screenId) {
    document.querySelectorAll('#gameView .screen').forEach(screen => {
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
        const response = await fetch(`${API_BASE_URL}/game`, {
            method: 'POST',
            headers: { 'Content-Type': 'application/json' },
            body: JSON.stringify({ name: creatorName })
        });
        if (!response.ok) throw new Error('Failed to create game');

        const { game, player } = await response.json();
        gameCode = game.code;
        playerId = player.id;
        
        hideModal('createGameModal');
        history.pushState(null, '', `/game/${gameCode}`);
        connectToGameStream();
        showGameView();
        showScreen('gameLobby');

    } catch (error) {
        console.error('Error creating game:', error);
        alert('Could not create game. Please try again.');
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
        const response = await fetch(`${API_BASE_URL}/game/${gameId}/join`, {
            method: 'POST',
            headers: { 'Content-Type': 'application/json' },
            body: JSON.stringify({ name: playerName })
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
            throw new Error('Failed to join game');
        }

        const { game, player } = await response.json();
        gameCode = game.code;
        playerId = player.id;

        hideModal('joinGameModal');
        history.pushState(null, '', `/game/${gameCode}`);
        connectToGameStream();
        showGameView();
        showScreen('gameLobby');

    } catch (error) {
        console.error('Error joining game:', error);
        alert('Could not join game. Please try again.');
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
        alert('Connection to the game has been lost.');
        leaveGame();
    };
}


function updateGameState(game, players) {
    const me = players.find(p => p.id === playerId);
    
    if (!me) {
        alert("You are no longer in the game.");
        leaveGame();
        return;
    }

    if (game.status === 'LOBBY') {
        updateLobby(game, players);
        showScreen('gameLobby');
    } else if (game.status === 'ACTIVE') {
        if (me.is_alive) {
            updateGameScreen(game, players, me);
            showScreen('gamePlaying');
        } else {
            const killer = players.find(p => p.id === me.killed_by);
            updateDeathScreen(killer);
            showScreen('deathScreen');
        }
    } else if (game.status === 'FINISHED') {
        const winner = players.find(p => p.is_alive);
        updateFinishedScreen(winner);
        showScreen('gameFinished');
    }
}

function updateLobby(game, players) {
    document.getElementById('lobbyGameName').textContent = `Lobby for ${game.code}`;
    document.getElementById('gameViewTitle').textContent = `Hitman Lobby - ${game.code}`;

    const playerList = document.getElementById('playerList');
    playerList.innerHTML = '';
    players.forEach(p => {
        const item = document.createElement('li');
        if (p.id === playerId) {
            item.style.fontWeight = 'bold';
        }
        item.textContent = p.name + (p.is_creator ? ' (Creator)' : '');
        playerList.appendChild(item);
    });
    
    const me = players.find(p => p.id === playerId);
    const startGameBtn = document.getElementById('startGameBtn');
    if (me && me.is_creator) {
        startGameBtn.style.display = 'block';
    } else {
        startGameBtn.style.display = 'none';
    }
    
    const shareUrl = `${window.location.origin}/game/${game.code}`;
    const shareLinkInput = document.getElementById('shareLink');
    shareLinkInput.value = shareUrl;
    
    const copyBtn = document.getElementById('copyLinkBtn');
    copyBtn.onclick = () => copyToClipboard(shareUrl, 'Link copied!');
    
    const qrCodeImg = document.getElementById('qrCode');
    qrCodeImg.src = `https://api.qrserver.com/v1/create-qr-code/?size=110x110&data=${encodeURIComponent(shareUrl)}`;

}

function updateGameScreen(game, players, me) {
    document.getElementById('gameViewTitle').textContent = "Game in Progress";
    const target = players.find(p => p.id === me.target_id);

    const targetInfo = document.getElementById('targetInfo');
    if (target) {
        targetInfo.innerHTML = `<legend>Your Target</legend><p>Your target is: <strong>${target.name}</strong></p><p>Your secret code is: <strong>${me.assassination_code}</strong></p>`;
    } else {
         targetInfo.innerHTML = `<legend>Your Target</legend><p>Waiting for target...</p>`;
    }

    const gamePlayerList = document.getElementById('gamePlayerList');
    gamePlayerList.innerHTML = '';
    players.forEach(p => {
        const item = document.createElement('li');
        let text = p.name;
        if (p.id === playerId) {
            item.style.fontWeight = 'bold';
            text += ' (You)';
        }
        if (!p.is_alive) {
            item.style.textDecoration = 'line-through';
            item.style.opacity = '0.6';
            text += ' - Eliminated';
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
        winnerText.textContent = "The game has finished.";
    }
}

async function startGame() {
    try {
        const response = await fetch(`${API_BASE_URL}/game/${gameCode}/start`, { method: 'POST' });
        if (!response.ok) throw new Error('Failed to start game');
    } catch (error) {
        console.error('Error starting game:', error);
        alert('Could not start game. Please try again.');
    }
}

async function eliminateTarget() {
    const assassinationCode = document.getElementById('assassinationCode').value;
    if (!assassinationCode) {
        alert('Please enter a code.');
        return;
    }

    try {
        const response = await fetch(`${API_BASE_URL}/game/${gameCode}/kill`, {
            method: 'POST',
            headers: { 'Content-Type': 'application/json' },
            body: JSON.stringify({ assassination_code: assassinationCode })
        });

        if (response.status === 400) {
            const { message } = await response.json();
            alert(`Failed: ${message}`);
        } else if (!response.ok) {
            throw new Error('Failed to eliminate target');
        } else {
            document.getElementById('assassinationCode').value = '';
            alert('Success! Target eliminated.');
        }
    } catch (error) {
        console.error('Error eliminating target:', error);
        alert('Could not process assassination. Please try again.');
    }
}

function leaveGame() {
    if (eventSource) {
        eventSource.close();
        eventSource = null;
    }
    gameCode = null;
    playerId = null;
    
    document.getElementById('gameView').style.display = 'none';
    document.getElementById('welcomeView').style.display = 'block';

    history.pushState(null, '', '/');
}

function copyToClipboard(text, successMessage) {
    navigator.clipboard.writeText(text).then(() => {
        if (successMessage) {
            alert(successMessage);
        }
    }).catch(err => {
        console.error('Failed to copy text: ', err);
        alert('Could not copy link to clipboard.');
    });
}

// Add a handler for the back/forward buttons
window.addEventListener('popstate', (event) => {
    // If we are in a game, leaving the page via back button should take us home.
    if (document.getElementById('gameView').style.display === 'block') {
        leaveGame();
    }
});
