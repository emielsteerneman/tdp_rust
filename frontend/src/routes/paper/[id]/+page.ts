import type { PageLoad } from './$types';
import { error } from '@sveltejs/kit';
import { getTeamInfo } from '$lib/api';
import type { RegistryEntry } from '$lib/api';

export const load: PageLoad = async ({ params, fetch }) => {
  const paper_lyt = params.id;

  // Parse paper_lyt: league__year__team
  const parts = paper_lyt.split('__');
  const leagueMachine = parts[0] ?? '';
  const year = parseInt(parts[1], 10) || 0;
  const teamPrettyName = parts[2]?.replace(/_/g, ' ') ?? '';

  // Fetch markdown (required) + team registry (optional) in parallel
  const [rawMarkdown, teamEntries] = await Promise.all([
    fetch(`/tdps/${paper_lyt}.md`).then((r) => {
      if (!r.ok) throw error(r.status, 'Paper not found');
      return r.text();
    }),
    getTeamInfo(teamPrettyName, fetch).catch((): RegistryEntry[] => [])
  ]);

  fetch(`/api/papers/${encodeURIComponent(paper_lyt)}/open`, { method: 'POST' }).catch(() => {});

  return {
    rawMarkdown,
    paper_lyt,
    teamEntries,
    teamPrettyName,
    leagueMachine,
    year
  };
};
