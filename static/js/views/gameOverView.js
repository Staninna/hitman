function updateGameOverUI({ winner }) {
    document.getElementById('winnerName').textContent = winner ? winner.name : "Nobody";
}

function initGameOver(gameService) {
    document.querySelector('.title-bar-controls button[aria-label="Close"]')?.addEventListener('click', () => gameService.leave());
    document.getElementById('backToMenuBtn')?.addEventListener('click', () => gameService.leave());
}

export const gameOverView = {
    name: "gameOver",
    screenId: "gameOverScreen",
    init: initGameOver,
    update: updateGameOverUI,
}; 