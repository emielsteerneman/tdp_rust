import type { LayoutLoad } from './$types';
import { listPapers, listTeams, listLeagues, listYears } from '$lib/api';

export const load: LayoutLoad = async ({ fetch }) => {
	try {
		const [papers, teams, leagues, years] = await Promise.all([
			listPapers(fetch),
			listTeams(undefined, fetch),
			listLeagues(fetch),
			listYears(fetch)
		]);

		return {
			papers,
			teams,
			leagues,
			years
		};
	} catch (error) {
		console.error('Error loading initial data:', error);
		return {
			papers: [],
			teams: [],
			leagues: [],
			years: []
		};
	}
};
