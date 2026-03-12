<script lang="ts">
	import { page } from '$app/stores';
	import { goto } from '$app/navigation';
	import type { League, TeamName } from '$lib/types';

	interface Props {
		leagues: League[];
		teams: TeamName[];
	}

	let { leagues, teams }: Props = $props();

	let selectedLeagues = $derived($page.url.searchParams.getAll('league'));
	let selectedYears = $derived($page.url.searchParams.getAll('year'));
	let selectedTeams = $derived($page.url.searchParams.getAll('team'));

	let hasFilters = $derived(
		selectedLeagues.length > 0 || selectedYears.length > 0 || selectedTeams.length > 0
	);

	// Build lookup maps for pretty names
	let leagueNameMap = $derived(
		new Map(leagues.map(l => [l.name, l.name_pretty]))
	);
	let teamNameMap = $derived(
		new Map(teams.map(t => [t.name, t.name_pretty]))
	);

	function removeFilter(type: 'league' | 'year' | 'team', value: string) {
		const params = new URLSearchParams($page.url.searchParams);
		const current = params.getAll(type);
		params.delete(type);
		current.filter(v => v !== value).forEach(v => params.append(type, v));
		const qs = params.toString();
		goto(`${$page.url.pathname}${qs ? '?' + qs : ''}`, { replaceState: true, keepFocus: true });
	}

	function clearAllFilters() {
		const params = new URLSearchParams($page.url.searchParams);
		params.delete('league');
		params.delete('year');
		params.delete('team');
		const qs = params.toString();
		goto(`${$page.url.pathname}${qs ? '?' + qs : ''}`, { replaceState: true, keepFocus: true });
	}
</script>

{#if hasFilters}
	<div class="flex flex-wrap items-center gap-2 px-4 py-3 bg-blue-50/50 dark:bg-blue-950/30 border-b border-blue-100 dark:border-blue-900/50">
		<span class="text-xs font-medium text-gray-500 dark:text-gray-400 uppercase tracking-wide mr-1">Active:</span>

		{#each selectedLeagues as league}
			<button
				onclick={() => removeFilter('league', league)}
				class="inline-flex items-center gap-1 px-2.5 py-1 text-xs font-medium rounded-full bg-blue-100 dark:bg-blue-900/50 text-blue-800 dark:text-blue-300 hover:bg-blue-200 dark:hover:bg-blue-900/70 transition-colors group"
			>
				{leagueNameMap.get(league) ?? league}
				<svg class="w-3 h-3 text-blue-400 group-hover:text-blue-600 dark:group-hover:text-blue-200" fill="none" stroke="currentColor" viewBox="0 0 24 24">
					<path stroke-linecap="round" stroke-linejoin="round" stroke-width="2.5" d="M6 18L18 6M6 6l12 12" />
				</svg>
			</button>
		{/each}

		{#each selectedYears as year}
			<button
				onclick={() => removeFilter('year', year)}
				class="inline-flex items-center gap-1 px-2.5 py-1 text-xs font-medium rounded-full bg-green-100 dark:bg-green-900/50 text-green-800 dark:text-green-300 hover:bg-green-200 dark:hover:bg-green-900/70 transition-colors group"
			>
				{year}
				<svg class="w-3 h-3 text-green-400 group-hover:text-green-600 dark:group-hover:text-green-200" fill="none" stroke="currentColor" viewBox="0 0 24 24">
					<path stroke-linecap="round" stroke-linejoin="round" stroke-width="2.5" d="M6 18L18 6M6 6l12 12" />
				</svg>
			</button>
		{/each}

		{#each selectedTeams as team}
			<button
				onclick={() => removeFilter('team', team)}
				class="inline-flex items-center gap-1 px-2.5 py-1 text-xs font-medium rounded-full bg-purple-100 dark:bg-purple-900/50 text-purple-800 dark:text-purple-300 hover:bg-purple-200 dark:hover:bg-purple-900/70 transition-colors group"
			>
				{teamNameMap.get(team) ?? team}
				<svg class="w-3 h-3 text-purple-400 group-hover:text-purple-600 dark:group-hover:text-purple-200" fill="none" stroke="currentColor" viewBox="0 0 24 24">
					<path stroke-linecap="round" stroke-linejoin="round" stroke-width="2.5" d="M6 18L18 6M6 6l12 12" />
				</svg>
			</button>
		{/each}

		<button
			onclick={clearAllFilters}
			class="text-xs text-gray-500 dark:text-gray-400 hover:text-red-600 dark:hover:text-red-400 ml-1 transition-colors"
		>
			Clear all
		</button>
	</div>
{/if}
