import { gameState } from './state.js';
import { leaveGame, startGame } from './api.js';
import { initializePolling } from './common.js';
import { registerUpdater } from './uimanager.js';
import { copyToClipboard, showToast } from './ui.js';

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

    const leaveButton = document.querySelector('#lobbyActions button:first-child');
    if(leaveButton) leaveButton.addEventListener('click', leaveGame);

    const startButton = document.getElementById('startGameBtn');
    if (startButton) {
        startButton.addEventListener('click', () => startGameAndHandleErrors());
    }
    
    const closeButton = document.querySelector('.title-bar-controls button[aria-label="Close"]');
    if(closeButton) closeButton.addEventListener('click', leaveGame);

    registerUpdater('lobby', updateLobbyUI);
});

function updateLobbyUI(game, players) {
    const { playerId } = gameState.getGameDetails();
    document.getElementById('lobbyGameName').textContent = `Game Lobby: ${game.code}`;

    const shareLink = `${window.location.origin}/game/${game.code}`;
    const shareLinkInput = document.getElementById('shareLink');
    shareLinkInput.value = shareLink;
    document.getElementById('copyLinkBtn').onclick = () => copyToClipboard(shareLink, 'Game link copied to clipboard!');

    const rejoinLink = `${window.location.origin}/game/${game.code}/player/${gameState.getGameDetails().authToken}`;
    const rejoinLinkInput = document.getElementById('rejoinLink');
    rejoinLinkInput.value = rejoinLink;
    document.getElementById('copyRejoinLinkBtn').onclick = () => copyToClipboard(rejoinLink, 'Rejoin link copied to clipboard!');

    const qrContainer = document.getElementById('qrCode');
    if (qrContainer && (!qrContainer.dataset.link || qrContainer.dataset.link !== shareLink)) {
        qrContainer.innerHTML = '';
        new QRCode(qrContainer, {
            text: shareLink,
            width: 110,
            height: 110,
            correctLevel: QRCode.CorrectLevel.H,
        });
        qrContainer.dataset.link = shareLink;
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
        if (players.length < 2) {
            startGameBtn.disabled = true;
            startGameBtn.title = 'Need at least 2 players to start the game';
        } else {
            startGameBtn.disabled = false;
            startGameBtn.title = '';
        }
    } else {
        startGameBtn.style.display = 'none';
    }
}

async function startGameAndHandleErrors() {
    try {
        await startGame();
    } catch (error) {
        showToast(error.message, 'error');
    }
} 