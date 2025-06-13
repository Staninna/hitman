import { gameState } from './state.js';
import { leaveGame, eliminateTarget as eliminateTargetApi } from './api.js';
import { initializePolling } from './common.js';
import { registerUpdater } from './uimanager.js';
import { showToast } from './ui.js';

document.addEventListener('DOMContentLoaded', () => {
    const serverContextElement = document.getElementById('server-context');
    if (!serverContextElement) return;

    const serverContext = JSON.parse(serverContextElement.textContent);

    if (serverContext.game_code && serverContext.auth_token && serverContext.player_id) {
        gameState.setGameDetails({
            gameCode: serverContext.game_code,
            playerId: serverContext.player_id,
            authToken: serverContext.auth_token,
        });

        initializePolling();
    }

    const assassinateButton = document.querySelector('#gamePlaying button');
    if(assassinateButton) assassinateButton.addEventListener('click', eliminateTarget);

    const closeButton = document.querySelector('.title-bar-controls button[aria-label="Close"]');
    if(closeButton) closeButton.addEventListener('click', leaveGame);

    registerUpdater('game', updateGameUI);
});

function updateGameUI(game, players, me) {
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
        if (p.id === gameState.getGameDetails().playerId) {
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
        showToast('Please enter your target\'s secret code.', 'error');
        return;
    }

    try {
        await eliminateTargetApi(code);
    } catch (error) {
        console.error('Error eliminating target:', error);
        showToast(`Could not eliminate target: ${error.message}`, 'error');
    }
} 