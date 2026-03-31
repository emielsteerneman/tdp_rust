import type {
	SearchResult,
	SearchParams,
	TDPName,
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
	if (params.paper_lyt_filter) {
		searchParams.append('paper_lyt_filter', params.paper_lyt_filter);
	}
	if (params.content_type_filter) {
		searchParams.append('content_type_filter', params.content_type_filter);
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

export async function submitSuggestion(message: string, fetchFn?: FetchFn): Promise<string> {
	return fetchApi<string>('/suggestion', fetchFn, {
		method: 'POST',
		body: JSON.stringify({ message })
	});
}

export interface TeamMetadataEntry {
	key: string;
	value: string;
	updated_at: string;
}

export interface PaperInfo {
	title: string;
	authors: { name: string; affiliation: string | null }[];
	institutions: string[];
	urls: string[];
}

export async function getPaperInfo(paper_lyt: string, fetchFn?: FetchFn): Promise<PaperInfo> {
	return fetchApi<PaperInfo>(`/papers/${encodeURIComponent(paper_lyt)}/info`, fetchFn);
}

export async function getTeamInfo(name: string, fetchFn?: FetchFn): Promise<TeamMetadataEntry[]> {
	return fetchApi<TeamMetadataEntry[]>(`/team-registry/${encodeURIComponent(name)}`, fetchFn);
}

export async function updateTeamInfo(
	team: string,
	code: string,
	entries: { key: string; value: string }[],
	fetchFn?: FetchFn
): Promise<string> {
	return fetchApi<string>('/team-registry', fetchFn, {
		method: 'POST',
		body: JSON.stringify({ team, code, entries })
	});
}
