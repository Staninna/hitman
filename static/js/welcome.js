import { showModal, hideModal, showToast } from './ui.js';
import { fetchApi } from './api.js';

document.addEventListener('DOMContentLoaded', () => {
    const serverContextElement = document.getElementById('server-context');
    let serverContext = {};
    if (serverContextElement) {
        serverContext = JSON.parse(serverContextElement.textContent);
    }

    document.getElementById('createGameBtn').addEventListener('click', () => showModal('createGameModal'));
    document.getElementById('joinGameBtn').addEventListener('click', () => showModal('joinGameModal'));

    document.getElementById('createGameModalClose').addEventListener('click', () => hideModal('createGameModal'));
    document.getElementById('createGameCancel').addEventListener('click', () => hideModal('createGameModal'));

    document.getElementById('joinGameModalClose').addEventListener('click', () => hideModal('joinGameModal'));
    document.getElementById('joinGameCancel').addEventListener('click', () => hideModal('joinGameModal'));

    if (serverContext.is_game_page && serverContext.game_exists) {
        document.getElementById('gameId').value = serverContext.game_code;
        showModal('joinGameModal');
    }

    document.getElementById('createGameConfirm').addEventListener('click', async () => {
        const creatorName = document.getElementById('creatorName').value;
        if (!creatorName) {
            showToast('Please enter your name.', 'error');
            return;
        }

        try {
            const data = await fetchApi('/api/game', {
                method: 'POST',
                body: JSON.stringify({ player_name: creatorName }),
            });
            window.location.href = `/game/${data.game_code}/player/${data.auth_token}/lobby`;
        } catch (error) {
            showToast(error.message, 'error');
        }
    });

    document.getElementById('joinGameConfirm').addEventListener('click', async () => {
        const gameId = document.getElementById('gameId').value;
        const playerName = document.getElementById('playerName').value;

        if (!gameId || !playerName) {
            showToast('Please enter both game ID and your name.', 'error');
            return;
        }

        try {
            const data = await fetchApi(`/api/game/${gameId}/join`, {
                method: 'POST',
                body: JSON.stringify({ player_name: playerName }),
            });
            window.location.href = `/game/${gameId}/player/${data.auth_token}/lobby`;
        } catch (error) {
            showToast(error.message, 'error');
        }
    });
}); 