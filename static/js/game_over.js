document.addEventListener('DOMContentLoaded', () => {
    const serverContextElement = document.getElementById('server-context');
    if (!serverContextElement) {
        return;
    }

    const serverContext = JSON.parse(serverContextElement.textContent);

    if (serverContext.game_code && serverContext.auth_token && serverContext.player_id) {
        gameCode = serverContext.game_code;
        playerId = serverContext.player_id;
        authToken = serverContext.auth_token;

        connectToGameStream();
    }

    const backToMenuButtonFinished = document.querySelector('#gameFinished button');
    if(backToMenuButtonFinished) backToMenuButtonFinished.addEventListener('click', leaveGame);

    const closeButton = document.querySelector('.title-bar-controls button[aria-label="Close"]');
    if(closeButton) closeButton.addEventListener('click', leaveGame);
});

function updateFinishedScreen(winner) {
    document.getElementById('gameViewTitle').textContent = "Game Over!";
    const winnerInfo = document.getElementById('winnerText');
    if (winner) {
        winnerInfo.innerHTML = `<strong>${winner.name}</strong> has won the game!`;
    } else {
        winnerInfo.innerHTML = `The game has finished!`;
    }
} 