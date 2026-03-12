import type { PageLoad } from './$types';
import { search } from '$lib/api';
import type { SearchParams, League, TeamName } from '$lib/types';

export const load: PageLoad = async ({ url, fetch, parent }) => {
	const query = url.searchParams.get('q') || '';

	if (!query) {
		return {
			searchResult: null,
			query: '',
			loading: false
		};
	}

	// Get layout data for name translation (machine name -> pretty name)
	const layoutData = await parent();

	// Read multi-value filter params (machine names from URL)
	const leagueMachineNames = url.searchParams.getAll('league');
	const yearStrings = url.searchParams.getAll('year');
	const teamMachineNames = url.searchParams.getAll('team');

	// Translate machine names to pretty names for the backend API
	const leaguePrettyNames = leagueMachineNames
		.map((name: string) => (layoutData.leagues as League[]).find((l: League) => l.name === name)?.name_pretty)
		.filter((n): n is string => n !== undefined);

	const teamPrettyNames = teamMachineNames
		.map((name: string) => (layoutData.teams as TeamName[]).find((t: TeamName) => t.name === name)?.name_pretty)
		.filter((n): n is string => n !== undefined);

	const params: SearchParams = {
		query,
		limit: 20,
		league_filter: leaguePrettyNames.length > 0 ? leaguePrettyNames.join(', ') : undefined,
		year_filter: yearStrings.length > 0 ? yearStrings.join(', ') : undefined,
		team_filter: teamPrettyNames.length > 0 ? teamPrettyNames.join(', ') : undefined,
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
