import { gameState } from "../core/state.js";
import { copyToClipboard } from "../utils/ui.js";

function updateLobbyUI({ game, players }) {
	const { playerId } = gameState.getGameDetails();
	document.getElementById("lobbyGameName").textContent = `Game Lobby: ${game.code}`;

	const shareLink = `${window.location.origin}/game/${game.code}`;
	document.getElementById("shareLink").value = shareLink;
	document.getElementById("copyLinkBtn").onclick = () =>
		copyToClipboard(shareLink, "Game link copied to clipboard!");

	const rejoinLink = `${window.location.origin}/game/${game.code}/player/${
		gameState.getGameDetails().authToken
	}`;
	document.getElementById("rejoinLink").value = rejoinLink;
	document.getElementById("copyRejoinLinkBtn").onclick = () =>
		copyToClipboard(rejoinLink, "Rejoin link copied to clipboard!");

	const qrContainer = document.getElementById("qrCode");
	if (
		qrContainer &&
		(!qrContainer.dataset.link || qrContainer.dataset.link !== shareLink)
	) {
		qrContainer.innerHTML = "";
		new QRCode(qrContainer, {
			text: shareLink,
			width: 110,
			height: 110,
			correctLevel: QRCode.CorrectLevel.H,
		});
		qrContainer.dataset.link = shareLink;
	}

	const playerList = document.getElementById("playerList");
	playerList.innerHTML = "";
	players.forEach((p) => {
		const li = document.createElement("li");
		li.textContent = `${p.name} ${p.id === game.host_id ? "(Host)" : ""}`;
		if (p.id === playerId) {
			li.style.fontWeight = "bold";
		}
		playerList.appendChild(li);
	});

	const me = players.find((p) => p.id === playerId);
	const startGameBtn = document.getElementById("startGameBtn");
	if (me && me.id === game.host_id) {
		startGameBtn.style.display = "block";
		startGameBtn.disabled = players.length < 2;
		startGameBtn.title =
			players.length < 2
				? "Need at least 2 players to start the game"
				: "";
	} else {
		startGameBtn.style.display = "none";
	}
}

function initLobby(gameService) {
	document
		.getElementById("leaveGameBtn")
		?.addEventListener("click", () => gameService.leave());
	document
		.getElementById("startGameBtn")
		?.addEventListener("click", () => gameService.startGame());
	document
		.querySelector('.title-bar-controls button[aria-label="Close"]')
		?.addEventListener("click", () => gameService.leave());
}

export const lobbyView = {
	name: "lobby",
	screenId: "gameLobby",
	init: initLobby,
	update: updateLobbyUI,
}; 