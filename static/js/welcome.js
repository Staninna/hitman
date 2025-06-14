import { showModal, hideModal } from './utils/ui.js';
import { showToast } from './utils/toast.js';
import { createGame, joinGame } from './services/api.js';

document.addEventListener('DOMContentLoaded', () => {
    const serverContextElement = document.getElementById('server-context');
    let serverContext = {};
    if (serverContextElement && serverContextElement.textContent) {
        try {
            serverContext = JSON.parse(serverContextElement.textContent);
        } catch (e) {
            console.error("Could not parse server context:", e);
        }
    }

    document.getElementById('createGameBtn')?.addEventListener('click', () => showModal('createGameModal'));
    document.getElementById('joinGameBtn')?.addEventListener('click', () => showModal('joinGameModal'));

    document.getElementById('createGameModalClose')?.addEventListener('click', () => hideModal('createGameModal'));
    document.getElementById('createGameCancel')?.addEventListener('click', () => hideModal('createGameModal'));

    document.getElementById('joinGameModalClose')?.addEventListener('click', () => hideModal('joinGameModal'));
    document.getElementById('joinGameCancel')?.addEventListener('click', () => hideModal('joinGameModal'));

    if (serverContext.is_game_page && serverContext.game_exists) {
        const gameIdInput = document.getElementById('gameId');
        if (gameIdInput) {
            gameIdInput.value = serverContext.game_code;
        }
        showModal('joinGameModal');
    }

    document.getElementById('createGameConfirm')?.addEventListener('click', async () => {
        const creatorNameInput = document.getElementById('creatorName');
        const creatorName = creatorNameInput ? creatorNameInput.value : '';
        if (!creatorName) {
            showToast('Please enter your name.', 'error');
            return;
        }

        try {
            const data = await createGame(creatorName);
            window.location.href = `/game/${data.game_code}/player/${data.auth_token}/lobby`;
        } catch (error) {
            showToast(error.message, 'error');
        }
    });

    document.getElementById('joinGameConfirm')?.addEventListener('click', async () => {
        const gameIdInput = document.getElementById('gameId');
        const playerNameInput = document.getElementById('playerName');
        
        const gameId = gameIdInput ? gameIdInput.value : '';
        const playerName = playerNameInput ? playerNameInput.value : '';

        if (!gameId || !playerName) {
            showToast('Please enter both game ID and your name.', 'error');
            return;
        }

        try {
            const data = await joinGame(gameId, playerName);
            window.location.href = `/game/${gameId}/player/${data.auth_token}/lobby`;
        } catch (error) {
            showToast(error.message, 'error');
        }
    });
}); 