import type { PageServerLoad, EntryGenerator } from './$types';
import { error } from '@sveltejs/kit';
import { API_ORIGIN } from '$lib/config';
import { getTeamInfo, listPapers } from '$lib/api';
import type { RegistryEntry } from '$lib/api';
import { paperTitle, metaDescription } from '$lib/seo';

export const prerender = true;

// Tell the prerenderer every paper page to generate.
export const entries: EntryGenerator = async () => {
	const papers = await listPapers(fetch);
	return papers.map((p) => ({
		id: `${p.league.name}__${p.year}__${p.team_name.name}`
	}));
};

export const load: PageServerLoad = async ({ params, fetch }) => {
	const paper_lyt = params.id;

	const parts = paper_lyt.split('__');
	const leagueMachine = parts[0] ?? '';
	const year = parseInt(parts[1], 10) || 0;
	const teamPrettyName = parts[2]?.replace(/_/g, ' ') ?? '';

	const [rawMarkdown, teamEntries] = await Promise.all([
		fetch(`${API_ORIGIN}/tdps/${paper_lyt}.md`).then((r) => {
			if (!r.ok) throw error(r.status, 'Paper not found');
			return r.text();
		}),
		getTeamInfo(teamPrettyName, fetch).catch((): RegistryEntry[] => [])
	]);

	const title = paperTitle(teamPrettyName, leagueMachine, year);
	const description = metaDescription(rawMarkdown, teamPrettyName, year);

	return { rawMarkdown, paper_lyt, teamEntries, teamPrettyName, leagueMachine, year, title, description };
};
