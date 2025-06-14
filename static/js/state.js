class GameStateManager {
    constructor() {
        this.gameCode = null;
        this.playerId = null;
        this.authToken = null;
        this.version = 0;
        this.pollingInterval = null;
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
            version: this.version,
        };
    }

    setPollingInterval(interval) {
        this.pollingInterval = interval;
    }

    getPollingInterval() {
        return this.pollingInterval;
    }

    clearPollingInterval() {
        if (this.pollingInterval) {
            clearInterval(this.pollingInterval);
            this.pollingInterval = null;
        }
    }

    setVersion(version) {
        this.version = version;
    }

    getVersion() {
        return this.version;
    }
}

export const gameState = new GameStateManager(); 