import type {
	SearchResult,
	SearchParams,
	TDPName,
	Paper,
	TeamName,
	League
} from './types';

const API_BASE = '/api';

async function fetchApi<T>(endpoint: string, options?: RequestInit): Promise<T> {
	const response = await fetch(`${API_BASE}${endpoint}`, {
		headers: {
			'Content-Type': 'application/json',
			...options?.headers
		},
		...options
	});

	if (!response.ok) {
		throw new Error(`API error: ${response.status} ${response.statusText}`);
	}

	return response.json();
}

/**
 * Search for TDP chunks
 * GET /api/search
 */
export async function search(params: SearchParams): Promise<SearchResult> {
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

	return fetchApi<SearchResult>(`/search?${searchParams.toString()}`);
}

/**
 * List all available papers
 * GET /api/papers
 */
export async function listPapers(): Promise<TDPName[]> {
	return fetchApi<TDPName[]>('/papers');
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
	index: number = 0
): Promise<Paper> {
	const id = `${league}__${year}__${team}__${index}`;
	return fetchApi<Paper>(`/papers/${id}`);
}

/**
 * Alternative: Get paper content using GetTdpContentsArgs structure
 * GET /api/papers with query parameters
 */
export async function getPaperByParams(
	league: string,
	year: number,
	team: string
): Promise<Paper> {
	const searchParams = new URLSearchParams({
		league,
		year: year.toString(),
		team
	});
	return fetchApi<Paper>(`/paper?${searchParams.toString()}`);
}

/**
 * List all teams
 * GET /api/teams
 */
export async function listTeams(hint?: string): Promise<TeamName[]> {
	const searchParams = new URLSearchParams();
	if (hint) {
		searchParams.append('hint', hint);
	}
	const query = searchParams.toString();
	return fetchApi<TeamName[]>(`/teams${query ? '?' + query : ''}`);
}

/**
 * List all leagues
 * GET /api/leagues
 */
export async function listLeagues(): Promise<League[]> {
	return fetchApi<League[]>('/leagues');
}

/**
 * List all years
 * GET /api/years
 */
export async function listYears(): Promise<number[]> {
	return fetchApi<number[]>('/years');
}
