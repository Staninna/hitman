document.addEventListener('DOMContentLoaded', () => {
    const serverContextElement = document.getElementById('server-context');
    if (!serverContextElement) {
        return;
    }

    const serverContext = JSON.parse(serverContextElement.textContent);

    if (serverContext.game_code && serverContext.auth_token && serverContext.player_id) {
        gameCode = serverContext.game_code;
        playerId = serverContext.player_id;
        authToken = serverContext.auth_token;

        connectToGameStream();
    }

    const leaveButton = document.querySelector('#lobbyActions button:first-child');
    if(leaveButton) leaveButton.addEventListener('click', leaveGame);

    const startGameButton = document.getElementById('startGameBtn');
    if(startGameButton) startGameButton.addEventListener('click', startGame);
    
    const closeButton = document.querySelector('.title-bar-controls button[aria-label="Close"]');
    if(closeButton) closeButton.addEventListener('click', leaveGame);
});

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
                const errorJson = JSON.parse(errorText);
                throw new Error(errorJson.message || 'Failed to start game');
            } catch (e) {
                throw new Error(errorText || 'Failed to start game');
            }
        }
    } catch (error) {
        console.error('Error starting game:', error);
        alert(`Could not start game: ${error.message}`);
    }
} 