import type {
	SearchResult,
	SearchParams,
	TDPName,
	Paper,
	TeamName,
	League
} from './types';

const API_BASE = '/api';

type FetchFn = typeof globalThis.fetch;

async function fetchApi<T>(endpoint: string, fetchFn: FetchFn = fetch, options?: RequestInit): Promise<T> {
	const response = await fetchFn(`${API_BASE}${endpoint}`, {
		headers: {
			'Content-Type': 'application/json',
			...options?.headers
		},
		...options
	});

	if (!response.ok) {
		throw new Error(`API error: ${response.status} ${response.statusText}`);
	}

	const json = await response.json();
	// API wraps responses in { data: ... }
	return json.data !== undefined ? json.data : json;
}

/**
 * Search for TDP chunks
 * GET /api/search
 */
export async function search(params: SearchParams, fetchFn?: FetchFn): Promise<SearchResult> {
	const searchParams = new URLSearchParams();
	searchParams.append('query', params.query);

	if (params.limit !== undefined) {
		searchParams.append('limit', params.limit.toString());
	}
	if (params.league_filter) {
		searchParams.append('league_filter', params.league_filter);
	}
	if (params.year_filter) {
		searchParams.append('year_filter', params.year_filter);
	}
	if (params.team_filter) {
		searchParams.append('team_filter', params.team_filter);
	}
	if (params.lyti_filter) {
		searchParams.append('lyti_filter', params.lyti_filter);
	}
	searchParams.append('search_type', params.search_type ?? 'hybrid');

	return fetchApi<SearchResult>(`/search?${searchParams.toString()}`, fetchFn);
}

/**
 * List all available papers
 * GET /api/papers
 */
export async function listPapers(fetchFn?: FetchFn): Promise<TDPName[]> {
	return fetchApi<TDPName[]>('/papers', fetchFn);
}

/**
 * Get a specific paper's content by ID (league, year, team)
 * GET /api/papers/:id
 *
 * The ID is constructed as: league__year__team__index
 * For example: soccer_smallsize__2019__RoboTeam_Twente__0
 */
export async function getPaper(
	league: string,
	year: number,
	team: string,
	index: number = 0,
	fetchFn?: FetchFn
): Promise<Paper> {
	const id = `${league}__${year}__${team}__${index}`;
	return fetchApi<Paper>(`/papers/${id}`, fetchFn);
}

/**
 * Alternative: Get paper content using GetTdpContentsArgs structure
 * GET /api/papers with query parameters
 */
export async function getPaperByParams(
	league: string,
	year: number,
	team: string,
	fetchFn?: FetchFn
): Promise<Paper> {
	const searchParams = new URLSearchParams({
		league,
		year: year.toString(),
		team
	});
	return fetchApi<Paper>(`/paper?${searchParams.toString()}`, fetchFn);
}

/**
 * List all teams
 * GET /api/teams
 */
export async function listTeams(hint?: string, fetchFn?: FetchFn): Promise<TeamName[]> {
	const searchParams = new URLSearchParams();
	if (hint) {
		searchParams.append('hint', hint);
	}
	const query = searchParams.toString();
	return fetchApi<TeamName[]>(`/teams${query ? '?' + query : ''}`, fetchFn);
}

/**
 * List all leagues
 * GET /api/leagues
 */
export async function listLeagues(fetchFn?: FetchFn): Promise<League[]> {
	return fetchApi<League[]>('/leagues', fetchFn);
}

/**
 * List all years
 * GET /api/years
 */
export async function listYears(fetchFn?: FetchFn): Promise<number[]> {
	return fetchApi<number[]>('/years', fetchFn);
}
