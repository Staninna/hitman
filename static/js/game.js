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

    const assassinateButton = document.querySelector('#gamePlaying button');
    if(assassinateButton) assassinateButton.addEventListener('click', eliminateTarget);

    const closeButton = document.querySelector('.title-bar-controls button[aria-label="Close"]');
    if(closeButton) closeButton.addEventListener('click', leaveGame);
});

function updateGameScreen(game, players, me) {
    document.getElementById('gameViewTitle').textContent = "Game in Progress";

    document.getElementById('playerSecretCode').textContent = me.secret_code || '...';

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

async function eliminateTarget() {
    const code = document.getElementById('assassinationCode').value;
    if (!code) {
        alert('Please enter your target\'s secret code.');
        return;
    }

    try {
        const response = await fetch(`${API_BASE_URL}/api/game/${gameCode}/eliminate`, {
            method: 'POST',
            headers: {
                'Content-Type': 'application/json',
                'Authorization': `Bearer ${authToken}`
            },
            body: JSON.stringify({ assassination_code: code })
        });
        if (!response.ok) {
            const errorData = await response.json();
            throw new Error(errorData.message || 'Failed to eliminate target');
        }
    } catch (error) {
        console.error('Error eliminating target:', error);
        alert(`Could not eliminate target: ${error.message}`);
    }
} 