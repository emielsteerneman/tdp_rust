import type { PageLoad } from './$types';
import { search } from '$lib/api';
import type { SearchParams } from '$lib/types';

export const load: PageLoad = async ({ url, fetch }) => {
	const query = url.searchParams.get('q') || '';

	if (!query) {
		return {
			searchResult: null,
			query: '',
			loading: false
		};
	}

	// Read multi-value filter params (machine names from URL)
	const leagueNames = url.searchParams.getAll('league');
	const yearStrings = url.searchParams.getAll('year');
	const teamNames = url.searchParams.getAll('team');

	const params: SearchParams = {
		query,
		limit: 20,
		league_filter: leagueNames.length > 0 ? leagueNames.join(', ') : undefined,
		year_filter: yearStrings.length > 0 ? yearStrings.join(', ') : undefined,
		team_filter: teamNames.length > 0 ? teamNames.join(', ') : undefined,
		content_type_filter: 'text'
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
