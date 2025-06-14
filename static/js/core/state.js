class GameStateManager {
	constructor() {
		this.gameCode = null;
		this.playerId = null;
		this.authToken = null;
		this.version = 0;
	}

	setGameDetails({ gameCode, playerId, authToken }) {
		this.gameCode = gameCode;
		this.playerId = playerId;
		this.authToken = authToken;
	}

	getGameDetails() {
		return {
			gameCode: this.gameCode,
			playerId: this.playerId,
			authToken: this.authToken,
		};
	}

	setVersion(version) {
		this.version = version;
	}

	getVersion() {
		return this.version;
	}
}

export const gameState = new GameStateManager(); 