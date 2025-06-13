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

    const backToMenuButtonFinished = document.querySelector('#gameFinished button');
    if(backToMenuButtonFinished) backToMenuButtonFinished.addEventListener('click', leaveGame);

    const closeButton = document.querySelector('.title-bar-controls button[aria-label="Close"]');
    if(closeButton) closeButton.addEventListener('click', leaveGame);

    registerUpdater('gameOver', updateGameOverUI);
});

function updateGameOverUI(winner) {
    document.getElementById('gameViewTitle').textContent = "Game Over!";
    const winnerInfo = document.getElementById('winnerText');
    if (winner) {
        winnerInfo.innerHTML = `<strong>${winner.name}</strong> has won the game!`;
    } else {
        winnerInfo.innerHTML = `The game has finished!`;
    }
} 