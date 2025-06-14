import { gameState } from "../core/state.js";
import { showToast } from "../utils/ui.js";

let lastRenderedSecret = null;
let html5QrScanner = null;

function updateGameUI({ game, players, me }) {
	document.getElementById("gameViewTitle").textContent = "Game in Progress";
	document.getElementById("playerSecretCode").textContent =
		me.secret_code || "...";

	if (me.secret_code && me.secret_code !== lastRenderedSecret) {
		lastRenderedSecret = me.secret_code;
		renderSecretQr(me.secret_code);
	}

	const targetInfo = document.getElementById("targetInfo");
	targetInfo.innerHTML = me.target_name
		? `<legend>Your Target</legend><p>Your target is: <strong>${me.target_name}</strong></p>`
		: `<legend>Your Target</legend><p>Waiting for target...</p>`;

	const gamePlayerList = document.getElementById("gamePlayerList");
	gamePlayerList.innerHTML = "";
	players
		.filter((p) => p.is_alive)
		.forEach((p) => {
			const item = document.createElement("li");
			let text = p.name;
			if (p.id === gameState.getGameDetails().playerId) {
				item.style.fontWeight = "bold";
				text += " (You)";
			}
			item.textContent = text;
			gamePlayerList.appendChild(item);
		});
}

function initGame(gameService) {
	document
		.getElementById("assassinateBtn")
		?.addEventListener("click", () => {
			const code = document.getElementById("assassinationCode").value;
			if (!code) {
				showToast("Please enter your target's secret code.", "error");
				return;
			}
			gameService.eliminateTarget(code);
		});

	document
		.getElementById("scanQrButton")
		?.addEventListener("click", () => startQrScan(gameService));
	document
		.getElementById("closeScannerBtn")
		?.addEventListener("click", stopQrScan);
	document
		.querySelector('.title-bar-controls button[aria-label="Close"]')
		?.addEventListener("click", () => gameService.leave());
}

function renderSecretQr(secret) {
    const container = document.getElementById('qrCode');
    if (!container) return;
    container.innerHTML = '';
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

function startQrScan(gameService) {
    if (!navigator.mediaDevices || !navigator.mediaDevices.getUserMedia) {
        showToast('Camera is not available in this browser.', 'error');
        return;
    }

    if (!window.isSecureContext && !['localhost', '127.0.0.1'].includes(location.hostname)) {
        showToast('Camera access requires HTTPS (or localhost). Please run the app over HTTPS.', 'error');
        return;
    }

    const overlay = document.getElementById('qrScannerOverlay');
    if (!overlay) return;
    overlay.style.display = 'flex';

    if (!html5QrScanner) {
        html5QrScanner = new Html5Qrcode('qrReader');
    }

	const successCallback = (decodedText) => onScanSuccess(decodedText, gameService);

    html5QrScanner
        .start(
            { facingMode: 'environment' },
            { fps: 10, qrbox: { width: 250, height: 250 } },
            successCallback,
            (err) => console.warn('QR scan error:', err),
        )
        .catch((err) => {
            console.error('Start scan failed:', err);
            showToast('Unable to start camera for scanning.', 'error');
            stopQrScan();
        });
}

function stopQrScan() {
    const overlay = document.getElementById('qrScannerOverlay');
    if (overlay) overlay.style.display = 'none';

    if (html5QrScanner && html5QrScanner.isScanning) {
        html5QrScanner.stop().catch(err => console.warn('Failed to stop scanner:', err));
    }
}

async function onScanSuccess(decodedText, gameService) {
	stopQrScan();
	await gameService.eliminateTarget(decodedText);
}

export const gameView = {
	name: "game",
	screenId: "gamePlaying",
	init: initGame,
	update: updateGameUI,
}; 