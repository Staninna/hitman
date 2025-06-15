import { gameState } from "../core/state.js";
import { showToast } from "../utils/ui.js";
import {
	startScanner,
	stopScanner,
} from "../core/qrScanner.js";

let lastRenderedSecret = null;

function renderSecretQr(secret) {
	const container = document.getElementById("qrCode");
	if (!container) return;
	container.innerHTML = "";
	try {
		new QRCode(container, {
			text: secret,
			width: 128,
			height: 128,
			correctLevel: QRCode.CorrectLevel.H,
		});
	} catch (err) {
		console.error("Failed to render QR code:", err);
	}
}

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

	document.getElementById("scanQrButton")?.addEventListener("click", () => {
		startScanner(async (decodedText) => {
			await gameService.eliminateTarget(decodedText);
		});
	});

	document
		.getElementById("closeScannerBtn")
		?.addEventListener("click", stopScanner);

	document
		.querySelector('.title-bar-controls button[aria-label="Close"]')
		?.addEventListener("click", () => gameService.leave());
}

export const gameView = {
	name: "game",
	screenId: "gamePlaying",
	init: initGame,
	update: updateGameUI,
};
