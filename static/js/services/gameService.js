import { gameState } from "../core/state.js";
import * as api from "./api.js";
import { showToast } from "../utils/ui.js";

export class GameService {
	#pollingIntervalId = null;

	constructor(gameDetails, viewManager) {
		this.viewManager = viewManager;
		gameState.setGameDetails(gameDetails);
	}

	async start() {
		this.stopPolling(); // Ensure no previous polling is running
		await this.fetchGameState();
		this.#pollingIntervalId = setInterval(
			() => this.pollForChanges(),
			2000,
		);
	}

	stopPolling() {
		if (this.#pollingIntervalId) {
			clearInterval(this.#pollingIntervalId);
			this.#pollingIntervalId = null;
		}
	}

	async pollForChanges() {
		const { gameCode } = gameState.getGameDetails();
		if (!gameCode) return;

		try {
			const data = await api.checkChanges(
				gameCode,
				gameState.getVersion(),
			);
			if (typeof data.current_version === "number") {
				gameState.setVersion(data.current_version);
			}
			if (data.changed) {
				await this.fetchGameState();
			}
			// Handle reconnection message if needed
		} catch (error) {
			console.error("Error polling for changes:", error);
			if (error.status === 404) {
				showToast(
					"This game session no longer exists. Returning to the home page.",
					"error",
				);
				setTimeout(() => this.leave(), 3000);
			} else {
				showToast(
					`Connection issue: ${error.message}. Will keep trying.`,
					"error",
				);
			}
		}
	}

	async fetchGameState() {
		const { gameCode } = gameState.getGameDetails();
		try {
			const { game, players, version } = await api.fetchGameState(
				gameCode,
			);
			if (typeof version === "number") {
				console.log("TODO: DO WE EVEN GET AN VERSION")
				gameState.setVersion(version);
			}
			// Dispatch the new state to the view manager
			this.viewManager.update(game, players);
		} catch (error) {
			console.error("Error fetching game state:", error);
			showToast(
				`Failed to update game state: ${error.message}`,
				"error",
			);
		}
	}

	async leave() {
		this.stopPolling();
		const { gameCode } = gameState.getGameDetails();
		if (gameCode) {
			try {
				await api.leaveGame(gameCode);
			} catch (error) {
				console.error("Error leaving game:", error);
				// Don't block redirect on failure
				// TODO: handle this better cuz otherwise other players are out of sync and an player is dangeling within the game
			}
		}
		window.location.href = "/";
	}

	async startGame() {
		const { gameCode } = gameState.getGameDetails();
		try {
			await api.startGame(gameCode);
		} catch (error) {
			showToast(error.message, "error");
		}
	}

	async eliminateTarget(secretCode) {
		const { gameCode } = gameState.getGameDetails();
		try {
			await api.eliminateTarget(gameCode, secretCode);
			showToast("Target elimination attempted!", "info");
		} catch (error) {
			showToast(error.message, "error");
		}
	}
} 