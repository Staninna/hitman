function updateEliminatedUI({ killer }) {
    document.getElementById('killerName').textContent = killer ? killer.name : "an unknown player";
}

function initEliminated(gameService) {
    document.querySelector('.title-bar-controls button[aria-label="Close"]')?.addEventListener('click', () => gameService.leave());
    document.getElementById('backToMenuBtn')?.addEventListener('click', () => gameService.leave());
}

export const eliminatedView = {
    name: "eliminated",
    screenId: "eliminatedScreen",
    init: initEliminated,
    update: updateEliminatedUI,
}; 