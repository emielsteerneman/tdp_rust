import type { PageLoad } from './$types';
import { search } from '$lib/api';
import type { SearchParams } from '$lib/types';

export const load: PageLoad = async ({ url, fetch }) => {
	const query = url.searchParams.get('q') || '';
	const league = url.searchParams.get('league') || undefined;
	const year = url.searchParams.get('year') || undefined;
	const team = url.searchParams.get('team') || undefined;

	if (!query) {
		return {
			searchResult: null,
			query: '',
			loading: false
		};
	}

	const params: SearchParams = {
		query,
		league_filter: league,
		year_filter: year,
		team_filter: team
	};

	try {
		const searchResult = await search(params, fetch);
		return {
			searchResult,
			query,
			loading: false
		};
	} catch (error) {
		console.error('Search error:', error);
		return {
			searchResult: null,
			query,
			error: error instanceof Error ? error.message : 'Search failed',
			loading: false
		};
	}
};
