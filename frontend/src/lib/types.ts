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
}

export interface BreadcrumbEntry {
	content_seq: number;
	title: string;
}

export interface SearchResultChunk {
	paper_lyt: string;
	league: League;
	year: number;
	team: TeamName;
	content_seq: number;
	chunk_seq: number;
	content_type: string;
	title: string;
	text: string;
	score: number;
	breadcrumbs: BreadcrumbEntry[];
}

export interface SearchSuggestions {
	teams: string[];
	leagues: string[];
}

export interface SearchResult {
	query: string;
	filter: Filter | null;
	chunks: SearchResultChunk[];
	suggestions: SearchSuggestions;
	highlight_terms: string[];
}

export interface Filter {
	leagues?: string[];
	years?: number[];
	teams?: string[];
	paper_lyts?: string[];
}

export type EmbedType = 'dense' | 'sparse' | 'hybrid';

export interface SearchParams {
	query: string;
	limit?: number;
	league_filter?: string;
	year_filter?: string;
	team_filter?: string;
	paper_lyt_filter?: string;
	content_type_filter?: string;
	search_type?: EmbedType;
}

export interface ApiResponse<T> {
	data: T;
}

