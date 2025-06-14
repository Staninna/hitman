import { showScreen } from "../utils/ui.js";
import { gameState } from "./state.js";
import { route } from "./router.js";

export class ViewManager {
	#views = new Map();

	register(view) {
		if (view.name) {
			this.#views.set(view.name, view);
		}
	}

	getView(name) {
		return this.#views.get(name);
	}

	update(game, players) {
		const { playerId } = gameState.getGameDetails();
		const me = players.find((p) => p.id === playerId);

		if (!me) {
			alert("You have been removed from the game.");
			// The service will handle the leave logic
			return;
		}

		route(game, players, me);

		const gameStatus = game.status.toLowerCase();
		let view, viewData;

		if (gameStatus === "lobby") {
			view = this.getView("lobby");
			viewData = { game, players };
		} else if (gameStatus === "inprogress") {
			if (me.is_alive) {
				view = this.getView("game");
				viewData = { game, players, me };
			} else {
				const killer = players.find((p) => p.id === me.killed_by);
				view = this.getView("eliminated");
				viewData = { killer };
			}
		} else if (gameStatus === "finished") {
			const winner = players.find((p) => p.is_alive);
			view = this.getView("gameOver");
			viewData = { winner };
		}

		if (view) {
			showScreen(view.screenId);
			view.update(viewData);
		}
	}
} 