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

    const assassinateButton = document.querySelector('#assassinateBtn');
    if(assassinateButton) assassinateButton.addEventListener('click', eliminateTarget);

    // QR scan button
    const scanQrButton = document.getElementById('scanQrButton');
    if (scanQrButton) scanQrButton.addEventListener('click', startQrScan);

    // Close scanner button
    const closeScannerBtn = document.getElementById('closeScannerBtn');
    if (closeScannerBtn) closeScannerBtn.addEventListener('click', stopQrScan);

    const closeButton = document.querySelector('.title-bar-controls button[aria-label="Close"]');
    if(closeButton) closeButton.addEventListener('click', leaveGame);

    registerUpdater('game', updateGameUI);
});

// Keeps track whether we already rendered the QR code to avoid redundant work
let lastRenderedSecret = null;
let html5QrScanner = null;
let resizeInterval = null;

function updateGameUI(game, players, me) {
    document.getElementById('gameViewTitle').textContent = "Game in Progress";

    document.getElementById('playerSecretCode').textContent = me.secret_code || '...';

    // Generate QR code for the player's own secret_code
    if (me.secret_code && me.secret_code !== lastRenderedSecret) {
        lastRenderedSecret = me.secret_code;
        renderSecretQr(me.secret_code);
    }

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

function renderSecretQr(secret) {
    const container = document.getElementById('qrCode');
    if (!container) return;
    container.innerHTML = '';
    /* global QRCode */
    try {
        new QRCode(container, {
            text: secret,
            width: 128,
            height: 128,
            correctLevel: QRCode.CorrectLevel.H,
        });
    } catch (err) {
        console.error('Failed to render QR code:', err);
    }
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
        showToast(error.message, 'error');
    }
}

// ================= QR Scanner =================

function startQrScan() {
    // Check if camera access is allowed in current context
    if (!navigator.mediaDevices || !navigator.mediaDevices.getUserMedia) {
        showToast('Camera is not available in this browser.', 'error');
        return;
    }

    // Camera access is only permitted in secure contexts (HTTPS or localhost)
    if (!window.isSecureContext && !['localhost', '127.0.0.1'].includes(location.hostname)) {
        showToast('Camera access requires HTTPS (or localhost). Please run the app over HTTPS.', 'error');
        return;
    }

    const overlay = document.getElementById('qrScannerOverlay');
    if (!overlay) return;
    overlay.style.display = 'flex';

    // Initialize scanner only once
    if (!html5QrScanner) {
        html5QrScanner = new Html5Qrcode('qrReader');
    }

    html5QrScanner
        .start(
            { facingMode: 'environment' },
            { fps: 10, qrbox: 250 },
            onScanSuccess,
            err => console.warn('QR scan error:', err)
        )
        .catch(err => {
            console.error('Start scan failed:', err);
            showToast('Unable to start camera for scanning.', 'error');
            stopQrScan();
        });
}

function stopQrScan() {
    const overlay = document.getElementById('qrScannerOverlay');
    if (overlay) overlay.style.display = 'none';

    if (html5QrScanner) {
        html5QrScanner.stop().catch(err => console.warn('Failed to stop scanner:', err));
    }

    window.removeEventListener('resize', adjustReaderSize);
    if (resizeInterval) {
        clearInterval(resizeInterval);
        resizeInterval = null;
    }
}

async function onScanSuccess(decodedText) {
    stopQrScan();
    try {
        await eliminateTargetApi(decodedText);
    } catch (error) {
        console.error('Error eliminating target:', error);
        showToast(error.message, 'error');
        return;
    }
    showToast('Target elimination attempted!', 'info');
} 