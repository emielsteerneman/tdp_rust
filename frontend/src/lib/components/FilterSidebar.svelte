<script lang="ts">
	import { page } from '$app/stores';
	import { goto } from '$app/navigation';
	import { slide } from 'svelte/transition';
	import type { League, TeamName } from '$lib/types';
	import { getLeagueColor } from '$lib/leagueColors';

	interface Props {
		leagues: League[];
		years: number[];
		teams: TeamName[];
	}

	let { leagues, years, teams }: Props = $props();

	// Section collapse state
	let leagueExpanded = $state(true);
	let yearExpanded = $state(true);
	let teamExpanded = $state(true);

	// Mobile drawer state
	let mobileDrawerOpen = $state(false);

	// Team search filter
	let teamSearchQuery = $state('');

	// Parse URL search params to get selected filters
	let selectedLeagues = $derived(
		$page.url.searchParams.getAll('league')
	);
	let selectedYears = $derived(
		$page.url.searchParams.getAll('year').map(y => parseInt(y))
	);
	let selectedTeams = $derived(
		$page.url.searchParams.getAll('team')
	);

	// Filtered teams based on search query, selected float to top
	let sortedFilteredTeams = $derived.by(() => {
		const filtered = teamSearchQuery.trim() === ''
			? teams
			: teams.filter(t =>
				t.name_pretty.toLowerCase().includes(teamSearchQuery.toLowerCase())
			);
		const selected = filtered.filter(t => selectedTeams.includes(t.name));
		const unselected = filtered.filter(t => !selectedTeams.includes(t.name));
		return [...selected, ...unselected];
	});

	// Sort years in descending order
	let sortedYears = $derived([...years].sort((a, b) => b - a));

	// Total active filter count (for mobile FAB badge)
	let totalActiveCount = $derived(
		selectedLeagues.length + selectedYears.length + selectedTeams.length
	);

	// Build hierarchical league tree grouped by major > minor > sub
	let leagueTree = $derived.by(() => {
		const majorMap = new Map<string, Map<string, League[]>>();
		for (const league of leagues) {
			if (!majorMap.has(league.league_major)) {
				majorMap.set(league.league_major, new Map());
			}
			const minorMap = majorMap.get(league.league_major)!;
			if (!minorMap.has(league.league_minor)) {
				minorMap.set(league.league_minor, []);
			}
			minorMap.get(league.league_minor)!.push(league);
		}

		return Array.from(majorMap.entries()).map(([major, minorMap]) => {
			const children = Array.from(minorMap.entries()).map(([minor, leagueList]) => {
				const hasSubs = leagueList.some(l => l.league_sub !== null);
				const allNames = leagueList.map(l => l.name);
				return {
					minorLabel: minor,
					league: !hasSubs ? leagueList[0] : null as League | null,
					subs: hasSubs
						? leagueList
							.filter(l => l.league_sub !== null)
							.map(l => ({ sub: l.league_sub!, league: l }))
						: [],
					allNames
				};
			});
			return {
				majorLabel: major,
				children,
				allNames: children.flatMap(c => c.allNames)
			};
		});
	});

	function toggleGroupLeagues(names: string[]) {
		const params = new URLSearchParams($page.url.searchParams);
		const current = params.getAll('league');
		const allSelected = names.every(n => current.includes(n));

		params.delete('league');
		if (allSelected) {
			current.filter(v => !names.includes(v)).forEach(v => params.append('league', v));
		} else {
			const merged = new Set([...current, ...names]);
			merged.forEach(v => params.append('league', v));
		}

		goto(`?${params.toString()}`, { replaceState: true, keepFocus: true });
	}

	function groupAllSelected(names: string[]): boolean {
		return names.length > 0 && names.every(n => selectedLeagues.includes(n));
	}

	function groupSomeSelected(names: string[]): boolean {
		return names.some(n => selectedLeagues.includes(n)) && !names.every(n => selectedLeagues.includes(n));
	}

	function toggleFilter(type: 'league' | 'year' | 'team', value: string) {
		const params = new URLSearchParams($page.url.searchParams);
		const currentValues = params.getAll(type);

		if (currentValues.includes(value)) {
			params.delete(type);
			currentValues.filter(v => v !== value).forEach(v => params.append(type, v));
		} else {
			params.append(type, value);
		}

		goto(`?${params.toString()}`, { replaceState: true, keepFocus: true });
	}

	function clearAllFilters() {
		const params = new URLSearchParams($page.url.searchParams);
		params.delete('league');
		params.delete('year');
		params.delete('team');
		const qs = params.toString();
		goto(`${$page.url.pathname}${qs ? '?' + qs : ''}`, { replaceState: true, keepFocus: true });
	}

	function toggleMobileDrawer() {
		mobileDrawerOpen = !mobileDrawerOpen;
	}
</script>

<!-- ==================== Shared snippets ==================== -->

{#snippet sectionHeader(label: string, count: number, expanded: boolean, toggle: () => void)}
	<button
		onclick={toggle}
		class="flex items-center justify-between w-full text-left font-medium text-gray-900 dark:text-gray-100 mb-2 group"
	>
		<span class="flex items-center gap-2">
			{label}
			{#if count > 0}
				<span class="inline-flex items-center justify-center min-w-5 h-5 px-1.5 text-xs font-medium rounded-full bg-blue-100 dark:bg-blue-900/50 text-blue-700 dark:text-blue-300">
					{count}
				</span>
			{/if}
		</span>
		<svg
			class="w-4 h-4 text-gray-400 dark:text-gray-500 group-hover:text-gray-600 dark:group-hover:text-gray-300 transition-transform duration-200 {expanded ? 'rotate-180' : ''}"
			fill="none"
			stroke="currentColor"
			viewBox="0 0 24 24"
		>
			<path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M19 9l-7 7-7-7" />
		</svg>
	</button>
{/snippet}

{#snippet leagueContent()}
	<div class="space-y-0.5">
		{#each leagueTree as group}
			<label class="flex items-center gap-2 cursor-pointer rounded-md px-2 py-1.5 hover:bg-gray-50 dark:hover:bg-gray-800 transition-colors mt-1">
				<input
					type="checkbox"
					checked={groupAllSelected(group.allNames)}
					indeterminate={groupSomeSelected(group.allNames)}
					onchange={() => toggleGroupLeagues(group.allNames)}
					class="w-4 h-4 rounded border-gray-300 dark:border-gray-600 text-blue-600 focus:ring-blue-500 focus:ring-offset-0 dark:bg-gray-700"
				/>
				<span class="w-2.5 h-2.5 rounded-full flex-shrink-0 {getLeagueColor(group.majorLabel).dot}"></span>
				<span class="text-sm font-semibold text-gray-900 dark:text-gray-100">{group.majorLabel}</span>
			</label>
			{#each group.children as child}
				{#if child.league}
					<label class="flex items-center gap-2 cursor-pointer rounded-md px-2 py-1 pl-6 transition-colors {selectedLeagues.includes(child.league.name) ? 'bg-blue-50 dark:bg-blue-900/30' : 'hover:bg-gray-50 dark:hover:bg-gray-800'}">
						<input
							type="checkbox"
							checked={selectedLeagues.includes(child.league.name)}
							onchange={() => toggleFilter('league', child.league!.name)}
							class="w-4 h-4 rounded border-gray-300 dark:border-gray-600 text-blue-600 focus:ring-blue-500 focus:ring-offset-0 dark:bg-gray-700"
						/>
						<span class="text-sm text-gray-700 dark:text-gray-300">{child.minorLabel}</span>
					</label>
				{:else}
					<label class="flex items-center gap-2 cursor-pointer rounded-md px-2 py-1 pl-6 hover:bg-gray-50 dark:hover:bg-gray-800 transition-colors mt-0.5">
						<input
							type="checkbox"
							checked={groupAllSelected(child.allNames)}
							indeterminate={groupSomeSelected(child.allNames)}
							onchange={() => toggleGroupLeagues(child.allNames)}
							class="w-4 h-4 rounded border-gray-300 dark:border-gray-600 text-blue-600 focus:ring-blue-500 focus:ring-offset-0 dark:bg-gray-700"
						/>
						<span class="text-sm font-medium text-gray-800 dark:text-gray-200">{child.minorLabel}</span>
					</label>
					{#each child.subs as sub}
						<label class="flex items-center gap-2 cursor-pointer rounded-md px-2 py-1 pl-10 transition-colors {selectedLeagues.includes(sub.league.name) ? 'bg-blue-50 dark:bg-blue-900/30' : 'hover:bg-gray-50 dark:hover:bg-gray-800'}">
							<input
								type="checkbox"
								checked={selectedLeagues.includes(sub.league.name)}
								onchange={() => toggleFilter('league', sub.league.name)}
								class="w-4 h-4 rounded border-gray-300 dark:border-gray-600 text-blue-600 focus:ring-blue-500 focus:ring-offset-0 dark:bg-gray-700"
							/>
							<span class="text-sm text-gray-700 dark:text-gray-300">{sub.sub}</span>
						</label>
					{/each}
				{/if}
			{/each}
		{/each}
	</div>
{/snippet}

{#snippet yearContent()}
	<div class="flex flex-wrap gap-1.5">
		{#each sortedYears as year}
			<button
				onclick={() => toggleFilter('year', year.toString())}
				class="px-2.5 py-1 text-sm rounded-full border transition-all duration-150
					{selectedYears.includes(year)
						? 'bg-blue-100 dark:bg-blue-900/50 border-blue-300 dark:border-blue-700 text-blue-800 dark:text-blue-300 font-medium shadow-sm'
						: 'bg-white dark:bg-gray-800 border-gray-200 dark:border-gray-700 text-gray-600 dark:text-gray-400 hover:border-blue-200 dark:hover:border-blue-700 hover:bg-blue-50 dark:hover:bg-blue-900/30'}"
			>
				{year}
			</button>
		{/each}
	</div>
{/snippet}

{#snippet teamContent()}
	<div>
		<div class="relative mb-2">
			<input
				type="text"
				bind:value={teamSearchQuery}
				placeholder="Search teams..."
				class="w-full pl-8 pr-3 py-1.5 text-sm border border-gray-200 dark:border-gray-700 rounded-lg focus:outline-none focus:ring-2 focus:ring-blue-500 focus:border-transparent bg-gray-50 dark:bg-gray-800 focus:bg-white dark:focus:bg-gray-700 text-gray-900 dark:text-gray-100 placeholder-gray-400 dark:placeholder-gray-500 transition-colors"
			/>
			<svg class="absolute left-2.5 top-1/2 -translate-y-1/2 w-3.5 h-3.5 text-gray-400 dark:text-gray-500" fill="none" stroke="currentColor" viewBox="0 0 24 24">
				<path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M21 21l-6-6m2-5a7 7 0 11-14 0 7 7 0 0114 0z" />
			</svg>
		</div>
		<div class="space-y-0.5 max-h-64 overflow-y-auto">
			{#each sortedFilteredTeams as team}
				<label class="flex items-center gap-2 cursor-pointer rounded-md px-2 py-1 transition-colors {selectedTeams.includes(team.name) ? 'bg-blue-50 dark:bg-blue-900/30' : 'hover:bg-gray-50 dark:hover:bg-gray-800'}">
					<input
						type="checkbox"
						checked={selectedTeams.includes(team.name)}
						onchange={() => toggleFilter('team', team.name)}
						class="w-4 h-4 rounded border-gray-300 dark:border-gray-600 text-blue-600 focus:ring-blue-500 focus:ring-offset-0 dark:bg-gray-700"
					/>
					<span class="text-sm text-gray-700 dark:text-gray-300 truncate">{team.name_pretty}</span>
				</label>
			{/each}
		</div>
	</div>
{/snippet}

{#snippet filterSections()}
	<!-- League Filter -->
	<div class="mb-5">
		{@render sectionHeader('League', selectedLeagues.length, leagueExpanded, () => leagueExpanded = !leagueExpanded)}
		{#if leagueExpanded}
			<div transition:slide={{ duration: 200 }}>
				{@render leagueContent()}
			</div>
		{/if}
	</div>

	<!-- Year Filter -->
	<div class="mb-5">
		{@render sectionHeader('Year', selectedYears.length, yearExpanded, () => yearExpanded = !yearExpanded)}
		{#if yearExpanded}
			<div transition:slide={{ duration: 200 }}>
				{@render yearContent()}
			</div>
		{/if}
	</div>

	<!-- Team Filter -->
	<div class="mb-5">
		{@render sectionHeader('Team', selectedTeams.length, teamExpanded, () => teamExpanded = !teamExpanded)}
		{#if teamExpanded}
			<div transition:slide={{ duration: 200 }}>
				{@render teamContent()}
			</div>
		{/if}
	</div>
{/snippet}

<!-- ==================== Mobile ==================== -->

<!-- Mobile Filter Toggle FAB -->
<button
	onclick={toggleMobileDrawer}
	class="lg:hidden fixed bottom-4 right-4 z-40 bg-blue-600 text-white rounded-full p-3.5 shadow-lg hover:bg-blue-700 active:scale-95 transition-all"
	aria-label="Toggle filters"
>
	<svg class="w-6 h-6" fill="none" stroke="currentColor" viewBox="0 0 24 24">
		<path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M3 4a1 1 0 011-1h16a1 1 0 011 1v2.586a1 1 0 01-.293.707l-6.414 6.414a1 1 0 00-.293.707V17l-4 4v-6.586a1 1 0 00-.293-.707L3.293 7.293A1 1 0 013 6.586V4z" />
	</svg>
	{#if totalActiveCount > 0}
		<span class="absolute -top-1 -right-1 inline-flex items-center justify-center w-5 h-5 text-xs font-bold text-white bg-red-500 rounded-full">
			{totalActiveCount}
		</span>
	{/if}
</button>

<!-- Mobile Drawer Overlay -->
{#if mobileDrawerOpen}
	<button
		type="button"
		class="lg:hidden fixed inset-0 bg-black/40 z-40 backdrop-blur-sm"
		onclick={toggleMobileDrawer}
		aria-label="Close filters"
		transition:slide={{ duration: 200 }}
	></button>
{/if}

<!-- Mobile Drawer -->
<aside class="lg:hidden fixed inset-y-0 left-0 z-50 w-80 bg-white dark:bg-gray-900 shadow-xl transform transition-transform duration-300 ease-in-out {mobileDrawerOpen ? 'translate-x-0' : '-translate-x-full'} overflow-y-auto">
	<div class="p-4">
		<div class="flex items-center justify-between mb-4">
			<h2 class="text-lg font-semibold text-gray-900 dark:text-gray-100">Filters</h2>
			<div class="flex items-center gap-2">
				{#if totalActiveCount > 0}
					<button
						onclick={clearAllFilters}
						class="text-xs font-medium text-red-600 dark:text-red-400 hover:text-red-800 dark:hover:text-red-300 transition-colors"
					>
						Clear all
					</button>
				{/if}
				<button
					onclick={toggleMobileDrawer}
					class="p-1.5 rounded-md text-gray-400 dark:text-gray-500 hover:text-gray-600 dark:hover:text-gray-300 hover:bg-gray-100 dark:hover:bg-gray-800 transition-colors"
					aria-label="Close filters"
				>
					<svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
						<path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 18L18 6M6 6l12 12" />
					</svg>
				</button>
			</div>
		</div>

		{@render filterSections()}
	</div>
</aside>

<!-- ==================== Desktop Sidebar ==================== -->

<aside class="hidden lg:block w-64 flex-shrink-0 border-r border-gray-200 dark:border-gray-800 h-screen sticky top-16 overflow-y-auto bg-gray-50/50 dark:bg-gray-900/50">
	<div class="p-4">
		<div class="flex items-center justify-between mb-4">
			<h2 class="text-sm font-semibold text-gray-500 dark:text-gray-400 uppercase tracking-wider">Filters</h2>
			{#if totalActiveCount > 0}
				<button
					onclick={clearAllFilters}
					class="text-xs font-medium text-red-600 dark:text-red-400 hover:text-red-800 dark:hover:text-red-300 transition-colors"
				>
					Clear all
				</button>
			{/if}
		</div>

		{@render filterSections()}
	</div>
</aside>
