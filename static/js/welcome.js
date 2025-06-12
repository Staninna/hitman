document.addEventListener('DOMContentLoaded', () => {
    // Welcome page buttons
    const createGameModalButton = document.querySelector('.window-body button:first-child');
    if (createGameModalButton && createGameModalButton.textContent.includes('Create New Game')) {
        createGameModalButton.addEventListener('click', () => showModal('createGameModal'));
    }

    const joinGameModalButton = document.querySelector('.window-body button:nth-child(2)');
    if (joinGameModalButton && joinGameModalButton.textContent.includes('Join Existing Game')) {
        joinGameModalButton.addEventListener('click', () => showModal('joinGameModal'));
    }

    // Create Game Modal
    const createGameModal = document.getElementById('createGameModal');
    if (createGameModal) {
        createGameModal.addEventListener('click', (event) => {
            // Only hide if clicking on the overlay itself
            if (event.target.id === 'createGameModal') {
                hideModal('createGameModal', event);
            }
        });
        const cancelButton = createGameModal.querySelector('button');
        cancelButton.addEventListener('click', () => hideModal('createGameModal'));

        const createButton = createGameModal.querySelector('button:nth-child(2)');
        createButton.addEventListener('click', createGame);
    }

    // Join Game Modal
    const joinGameModal = document.getElementById('joinGameModal');
    if (joinGameModal) {
        joinGameModal.addEventListener('click', (event) => {
             // Only hide if clicking on the overlay itself
            if (event.target.id === 'joinGameModal') {
                hideModal('joinGameModal', event);
            }
        });
        const cancelButton = joinGameModal.querySelector('button');
        cancelButton.addEventListener('click', () => hideModal('joinGameModal'));

        const joinButton = joinGameModal.querySelector('button:nth-child(2)');
        joinButton.addEventListener('click', joinGame);
    }
});

async function createGame() {
    const creatorName = document.getElementById('creatorName').value;
    if (!creatorName) {
        alert('Please enter your name.');
        return;
    }

    try {
        const response = await fetch(`${API_BASE_URL}/api/game`, {
            method: 'POST',
            headers: { 'Content-Type': 'application/json' },
            body: JSON.stringify({ player_name: creatorName })
        });
        if (!response.ok) {
            const error = await response.json();
            throw new Error(error.message || 'Failed to create game');
        }

        const { game_code, auth_token } = await response.json();
        window.location.href = `/game/${game_code}/player/${auth_token}`;
    } catch (error) {
        console.error('Error creating game:', error);
        alert(`Could not create game: ${error.message}`);
    }
}


async function joinGame() {
    const gameId = document.getElementById('gameId').value.trim();
    const playerName = document.getElementById('playerName').value.trim();

    if (!gameId || !playerName) {
        alert('Please enter both a game ID and your name.');
        return;
    }

    try {
        const response = await fetch(`${API_BASE_URL}/api/game/${gameId}/join`, {
            method: 'POST',
            headers: { 'Content-Type': 'application/json' },
            body: JSON.stringify({ player_name: playerName })
        });

        if (response.status === 404) {
            alert('Game not found.');
            return;
        }
        if (response.status === 409) {
            alert('A player with that name already exists in the game.');
            return;
        }
        if (!response.ok) {
            const error = await response.json();
            throw new Error(error.message || 'Failed to join game');
        }

        const { game_code, auth_token } = await response.json();
        window.location.href = `/game/${game_code}/player/${auth_token}`;
    } catch (error) {
        console.error('Error joining game:', error);
        alert(`Could not join game: ${error.message}`);
    }
} 