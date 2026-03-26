import type { PageLoad } from './$types';
import { error } from '@sveltejs/kit';
import { getPaperInfo, getTeamInfo } from '$lib/api';
import type { PaperInfo, TeamMetadataEntry } from '$lib/api';

export const load: PageLoad = async ({ params, fetch }) => {
  const lyti = params.id;

  // Parse lyti: league__year__team__index
  const parts = lyti.split('__');
  const leagueMachine = parts[0] ?? '';
  const year = parseInt(parts[1], 10) || 0;
  const teamPrettyName = parts[2]?.replace(/_/g, ' ') ?? '';

  // Fetch markdown (required) + paper info and team registry (optional) in parallel
  const [rawMarkdown, paperInfo, teamEntries] = await Promise.all([
    fetch(`/tdps/${lyti}.md`).then((r) => {
      if (!r.ok) throw error(r.status, 'Paper not found');
      return r.text();
    }),
    getPaperInfo(lyti, fetch).catch((): PaperInfo | null => null),
    getTeamInfo(teamPrettyName, fetch).catch((): TeamMetadataEntry[] => [])
  ]);

  fetch(`/api/papers/${encodeURIComponent(lyti)}/open`, { method: 'POST' }).catch(() => {});

  return {
    rawMarkdown,
    lyti,
    paperInfo,
    teamEntries,
    teamPrettyName,
    leagueMachine,
    year
  };
};
