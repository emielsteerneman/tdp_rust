import type { LayoutLoad } from './$types';
import { listPapers, listTeams, listLeagues, listYears } from '$lib/api';

export const load: LayoutLoad = async () => {
	try {
		const [papers, teams, leagues, years] = await Promise.all([
			listPapers(),
			listTeams(),
			listLeagues(),
			listYears()
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
