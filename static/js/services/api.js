import { gameState } from "../core/state.js";

const API_BASE_URL = `${window.location.protocol}//${window.location.host}`;

class ApiError extends Error {
	constructor(message, status) {
		super(message);
		this.name = "ApiError";
		this.status = status;
	}
}

async function fetchApi(url, options = {}) {
	const { authToken } = gameState.getGameDetails();

	const headers = {
		"Content-Type": "application/json",
		...options.headers,
	};

	if (authToken) {
		headers["Authorization"] = `Bearer ${authToken}`;
	}

	const response = await fetch(url, { ...options, headers });

	if (!response.ok) {
		let message = `API request failed with status ${response.status}`;
		try {
			const data = await response.clone().json();
			if (data && data.error) {
				message = data.error;
			}
		} catch (_) {
			// Ignore and fall back to default message
		}
		throw new ApiError(message, response.status);
	}

	return response.json();
}

// --- Game API Calls ---

export const createGame = (playerName) =>
	fetchApi("/api/game", {
		method: "POST",
		body: JSON.stringify({ player_name: playerName }),
	});

export const joinGame = (gameCode, playerName) =>
	fetchApi(`/api/game/${gameCode}/join`, {
		method: "POST",
		body: JSON.stringify({ player_name: playerName }),
	});

export const fetchGameState = (gameCode) =>
	fetchApi(`${API_BASE_URL}/api/game/${gameCode}`);

export const checkChanges = (gameCode, version) =>
	fetchApi(
		`${API_BASE_URL}/api/game/${gameCode}/changed?version=${version}`,
	);

export const leaveGame = (gameCode) =>
	fetchApi(`${API_BASE_URL}/api/game/${gameCode}/leave`, {
		method: "POST",
		body: JSON.stringify({}),
	});

export const startGame = (gameCode) =>
	fetchApi(`${API_BASE_URL}/api/game/${gameCode}/start`, {
		method: "POST",
		body: JSON.stringify({}),
	});

export const eliminateTarget = (gameCode, secretCode) =>
	fetchApi(`${API_BASE_URL}/api/game/${gameCode}/eliminate`, {
		method: "POST",
		body: JSON.stringify({ secret_code: secretCode }),
	}); 