import { showScreen } from './ui.js';
import { gameState } from "./state.js";
import { leaveGame } from "./api.js";
import { route } from "./router.js";

// These functions will be defined in other files and registered here
const updaters = {};

export function registerUpdater(screen, updater) {
    updaters[screen] = updater;
}

export function updateGameState(game, players) {
    const { playerId } = gameState.getGameDetails();
    const me = players.find(p => p.id === playerId);

    if (!me) {
        alert("You have been removed from the game.");
        leaveGame();
        return;
    }

    route(game, players, me);

    const gameStatus = game.status.toLowerCase();

    if (gameStatus === 'lobby') {
        showScreen('gameLobby');
        updaters.lobby?.(game, players);
    } else if (gameStatus === 'inprogress') {
        if (me.is_alive) {
            showScreen('gamePlaying');
            updaters.game?.(game, players, me);
        } else {
            const killer = players.find(p => p.id === me.killed_by);
            showScreen('deathScreen');
            updaters.eliminated?.(killer);
        }
    } else if (gameStatus === 'finished') {
        const winner = players.find(p => p.is_alive);
        showScreen('gameFinished');
        updaters.gameOver?.(winner);
    }
} 