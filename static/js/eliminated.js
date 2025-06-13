import { gameState } from './state.js';
import { leaveGame } from './api.js';
import { initializePolling } from './common.js';
import { registerUpdater } from './uimanager.js';

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
    
    const backToMenuButtonDeath = document.querySelector('#deathScreen button');
    if(backToMenuButtonDeath) backToMenuButtonDeath.addEventListener('click', leaveGame);

    const closeButton = document.querySelector('.title-bar-controls button[aria-label="Close"]');
    if(closeButton) closeButton.addEventListener('click', leaveGame);

    registerUpdater('eliminated', updateEliminatedUI);
});

function updateEliminatedUI(killer) {
    document.getElementById('gameViewTitle').textContent = "You've Been Eliminated";
    const killerInfo = document.getElementById('killerInfo');
    if (killer) {
        killerInfo.innerHTML = `<legend>Eliminated</legend><p>You were eliminated by: <strong>${killer.name}</strong></p>`;
    } else {
        killerInfo.innerHTML = `<legend>Eliminated</legend><p>You were eliminated.</p>`;
    }
} 