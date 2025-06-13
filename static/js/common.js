import { gameState } from './state.js';
import { pollForChanges, fetchGameState } from './api.js';

function startPolling() {
    stopPolling();
    fetchGameState();
    const interval = setInterval(pollForChanges, 2000);
    gameState.setPollingInterval(interval);
}

function stopPolling() {
    gameState.clearPollingInterval();
}

export function initializePolling() {
    startPolling();
} 