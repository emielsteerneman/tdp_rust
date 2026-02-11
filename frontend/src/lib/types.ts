// TypeScript interfaces mirroring Rust data structures

export interface League {
	league_major: string;
	league_minor: string;
	league_sub: string | null;
	name: string;
	name_pretty: string;
}

export interface TeamName {
	name: string;
	name_pretty: string;
}

export interface TDPName {
	league: League;
	team_name: TeamName;
	year: number;
	index: number;
}

export interface SearchResultChunk {
	league_year_team_idx: string;
	league: League;
	year: number;
	team: TeamName;
	paragraph_sequence_id: number;
	chunk_sequence_id: number;
	idx_begin: number;
	idx_end: number;
	text: string;
}

export interface ScoredChunk {
	chunk: SearchResultChunk;
	score: number;
}

export interface SearchSuggestions {
	teams: string[];
	leagues: string[];
}

export interface SearchResult {
	query: string;
	filter: Filter | null;
	chunks: ScoredChunk[];
	suggestions: SearchSuggestions;
}

export interface Filter {
	leagues?: string[];
	years?: number[];
	teams?: string[];
	league_year_team_indexes?: string[];
}

export type EmbedType = 'dense' | 'sparse' | 'hybrid';

export interface SearchParams {
	query: string;
	limit?: number;
	league_filter?: string;
	year_filter?: string;
	team_filter?: string;
	lyti_filter?: string;
	search_type?: EmbedType;
}

export interface ApiResponse<T> {
	data: T;
}

// For get_tdp_contents, the response is just a string (markdown)
export type Paper = string;
