import { GameService } from "./services/gameService.js";
import { ViewManager } from "./core/viewManager.js";
import { lobbyView } from "./views/lobbyView.js";
import { gameView } from "./views/gameView.js";
import { eliminatedView } from "./views/eliminatedView.js";
import { gameOverView } from "./views/gameOverView.js";

document.addEventListener("DOMContentLoaded", () => {
	const serverContextElement = document.getElementById("server-context");
	if (!serverContextElement) return;

	const serverContext = JSON.parse(serverContextElement.textContent);
	const { game_code, auth_token, player_id, page_name } = serverContext;

	if (!game_code || !auth_token || !player_id) {
		// If we are on a game page without context, something is wrong.
		// This could be handled more gracefully, e.g., redirecting home.
		console.error("Missing server context on a game page.");
		return;
	}

	// Initialize the View Manager with all available views
	const viewManager = new ViewManager();
	viewManager.register(lobbyView);
	viewManager.register(gameView);
	viewManager.register(eliminatedView);
	viewManager.register(gameOverView);

	// Initialize the Game Service
	const gameService = new GameService(
		{
			gameCode: game_code,
			playerId: player_id,
			authToken: auth_token,
		},
		viewManager,
	);

	// Initialize the specific view for the current page
	// This binds event listeners (like buttons) for the visible view.
	const currentView = viewManager.getView(page_name);
	if (currentView && currentView.init) {
		currentView.init(gameService);
	}

	// Start the game logic (fetches initial state and starts polling)
	gameService.start();
}); 