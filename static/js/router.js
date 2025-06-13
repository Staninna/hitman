import { gameState } from './state.js';

function buildPath(page) {
    const { gameCode, authToken } = gameState.getGameDetails();
    return `/game/${gameCode}/player/${authToken}/${page}`;
}

function handleRedirect(path) {
    if (window.location.pathname !== path) {
        window.location.href = path;
    }
}

export function route(game, players, me) {
    const gameStatus = game.status.toLowerCase();
    
    if (gameStatus === 'lobby') {
        handleRedirect(buildPath('lobby'));
    } else if (gameStatus === 'inprogress') {
        if (me.is_alive) {
            handleRedirect(buildPath('game'));
        } else {
            handleRedirect(buildPath('eliminated'));
        }
    } else if (gameStatus === 'finished') {
        handleRedirect(buildPath('game_over'));
    }
} 