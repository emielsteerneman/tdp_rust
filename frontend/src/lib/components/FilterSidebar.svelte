<script lang="ts">
	import { page } from '$app/stores';
	import { goto } from '$app/navigation';
	import type { League, TeamName } from '$lib/types';

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

	// Filtered teams based on search query
	let filteredTeams = $derived(
		teamSearchQuery.trim() === ''
			? teams
			: teams.filter(t =>
				t.name_pretty.toLowerCase().includes(teamSearchQuery.toLowerCase())
			)
	);

	// Sort years in descending order
	let sortedYears = $derived([...years].sort((a, b) => b - a));

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
			// Deselect all in this group, keep the rest
			current.filter(v => !names.includes(v)).forEach(v => params.append('league', v));
		} else {
			// Select all in this group, plus keep existing
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
			// Remove the value
			params.delete(type);
			currentValues.filter(v => v !== value).forEach(v => params.append(type, v));
		} else {
			// Add the value
			params.append(type, value);
		}

		goto(`?${params.toString()}`, { replaceState: true, keepFocus: true });
	}

	function clearAllFilters() {
		goto('/', { replaceState: true });
	}

	function hasActiveFilters() {
		return selectedLeagues.length > 0 || selectedYears.length > 0 || selectedTeams.length > 0;
	}

	function toggleMobileDrawer() {
		mobileDrawerOpen = !mobileDrawerOpen;
	}
</script>

{#snippet leagueTreeContent()}
	<div class="space-y-1">
		{#each leagueTree as group}
			<label class="flex items-center space-x-2 cursor-pointer mt-2">
				<input
					type="checkbox"
					checked={groupAllSelected(group.allNames)}
					indeterminate={groupSomeSelected(group.allNames)}
					onchange={() => toggleGroupLeagues(group.allNames)}
					class="w-4 h-4 text-blue-600 border-gray-300 rounded focus:ring-blue-500"
				/>
				<span class="text-sm font-semibold text-gray-900">{group.majorLabel}</span>
			</label>
			{#each group.children as child}
				{#if child.league}
					<label class="flex items-center space-x-2 cursor-pointer pl-4">
						<input
							type="checkbox"
							checked={selectedLeagues.includes(child.league.name)}
							onchange={() => toggleFilter('league', child.league!.name)}
							class="w-4 h-4 text-blue-600 border-gray-300 rounded focus:ring-blue-500"
						/>
						<span class="text-sm text-gray-700">{child.minorLabel}</span>
					</label>
				{:else}
					<label class="flex items-center space-x-2 cursor-pointer pl-4 mt-1">
						<input
							type="checkbox"
							checked={groupAllSelected(child.allNames)}
							indeterminate={groupSomeSelected(child.allNames)}
							onchange={() => toggleGroupLeagues(child.allNames)}
							class="w-4 h-4 text-blue-600 border-gray-300 rounded focus:ring-blue-500"
						/>
						<span class="text-sm font-medium text-gray-800">{child.minorLabel}</span>
					</label>
					{#each child.subs as sub}
						<label class="flex items-center space-x-2 cursor-pointer pl-8">
							<input
								type="checkbox"
								checked={selectedLeagues.includes(sub.league.name)}
								onchange={() => toggleFilter('league', sub.league.name)}
								class="w-4 h-4 text-blue-600 border-gray-300 rounded focus:ring-blue-500"
							/>
							<span class="text-sm text-gray-700">{sub.sub}</span>
						</label>
					{/each}
				{/if}
			{/each}
		{/each}
	</div>
{/snippet}

<!-- Mobile Filter Toggle Button -->
<button
	onclick={toggleMobileDrawer}
	class="lg:hidden fixed bottom-4 right-4 z-40 bg-blue-600 text-white rounded-full p-4 shadow-lg hover:bg-blue-700 transition-colors"
	aria-label="Toggle filters"
>
	<svg class="w-6 h-6" fill="none" stroke="currentColor" viewBox="0 0 24 24">
		<path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 6V4m0 2a2 2 0 100 4m0-4a2 2 0 110 4m-6 8a2 2 0 100-4m0 4a2 2 0 110-4m0 4v2m0-6V4m6 6v10m6-2a2 2 0 100-4m0 4a2 2 0 110-4m0 4v2m0-6V4" />
	</svg>
</button>

<!-- Mobile Drawer Overlay -->
{#if mobileDrawerOpen}
	<button
		type="button"
		class="lg:hidden fixed inset-0 bg-black bg-opacity-50 z-40"
		onclick={toggleMobileDrawer}
		aria-label="Close filters"
	></button>
{/if}

<!-- Mobile Drawer -->
<aside class="lg:hidden fixed inset-y-0 left-0 z-50 w-80 bg-white shadow-xl transform transition-transform duration-300 ease-in-out {mobileDrawerOpen ? 'translate-x-0' : '-translate-x-full'} overflow-y-auto">
	<div class="p-4">
		<div class="flex items-center justify-between mb-4">
			<h2 class="text-lg font-semibold text-gray-900">Filters</h2>
			<div class="flex items-center space-x-2">
				{#if hasActiveFilters()}
					<button
						onclick={clearAllFilters}
						class="text-sm text-blue-600 hover:text-blue-800"
					>
						Clear all
					</button>
				{/if}
				<button
					onclick={toggleMobileDrawer}
					class="p-1 text-gray-500 hover:text-gray-700"
					aria-label="Close filters"
				>
					<svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
						<path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 18L18 6M6 6l12 12" />
					</svg>
				</button>
			</div>
		</div>

		<!-- League Filter -->
		<div class="mb-6">
			<button
				onclick={() => leagueExpanded = !leagueExpanded}
				class="flex items-center justify-between w-full text-left font-medium text-gray-900 mb-2"
			>
				<span>League</span>
				<svg
					class="w-5 h-5 transition-transform {leagueExpanded ? 'rotate-180' : ''}"
					fill="none"
					stroke="currentColor"
					viewBox="0 0 24 24"
				>
					<path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M19 9l-7 7-7-7" />
				</svg>
			</button>
			{#if leagueExpanded}
				{@render leagueTreeContent()}
			{/if}
		</div>

		<!-- Year Filter -->
		<div class="mb-6">
			<button
				onclick={() => yearExpanded = !yearExpanded}
				class="flex items-center justify-between w-full text-left font-medium text-gray-900 mb-2"
			>
				<span>Year</span>
				<svg
					class="w-5 h-5 transition-transform {yearExpanded ? 'rotate-180' : ''}"
					fill="none"
					stroke="currentColor"
					viewBox="0 0 24 24"
				>
					<path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M19 9l-7 7-7-7" />
				</svg>
			</button>
			{#if yearExpanded}
				<div class="space-y-2">
					{#each sortedYears as year}
						<label class="flex items-center space-x-2 cursor-pointer">
							<input
								type="checkbox"
								checked={selectedYears.includes(year)}
								onchange={() => toggleFilter('year', year.toString())}
								class="w-4 h-4 text-blue-600 border-gray-300 rounded focus:ring-blue-500"
							/>
							<span class="text-sm text-gray-700">{year}</span>
						</label>
					{/each}
				</div>
			{/if}
		</div>

		<!-- Team Filter -->
		<div class="mb-6">
			<button
				onclick={() => teamExpanded = !teamExpanded}
				class="flex items-center justify-between w-full text-left font-medium text-gray-900 mb-2"
			>
				<span>Team</span>
				<svg
					class="w-5 h-5 transition-transform {teamExpanded ? 'rotate-180' : ''}"
					fill="none"
					stroke="currentColor"
					viewBox="0 0 24 24"
				>
					<path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M19 9l-7 7-7-7" />
				</svg>
			</button>
			{#if teamExpanded}
				<div class="space-y-2">
					<input
						type="text"
						bind:value={teamSearchQuery}
						placeholder="Search teams..."
						class="w-full px-3 py-1.5 text-sm border border-gray-300 rounded focus:outline-none focus:ring-2 focus:ring-blue-500 focus:border-transparent mb-2"
					/>
					<div class="space-y-2 max-h-64 overflow-y-auto">
						{#each filteredTeams as team}
							<label class="flex items-center space-x-2 cursor-pointer">
								<input
									type="checkbox"
									checked={selectedTeams.includes(team.name)}
									onchange={() => toggleFilter('team', team.name)}
									class="w-4 h-4 text-blue-600 border-gray-300 rounded focus:ring-blue-500"
								/>
								<span class="text-sm text-gray-700">{team.name_pretty}</span>
							</label>
						{/each}
					</div>
				</div>
			{/if}
		</div>

	</div>
</aside>

<!-- Desktop Sidebar -->
<aside class="hidden lg:block w-64 flex-shrink-0 bg-white border-r border-gray-200 h-screen sticky top-16 overflow-y-auto">
	<div class="p-4">
		<div class="flex items-center justify-between mb-4">
			<h2 class="text-lg font-semibold text-gray-900">Filters</h2>
			{#if hasActiveFilters()}
				<button
					onclick={clearAllFilters}
					class="text-sm text-blue-600 hover:text-blue-800"
				>
					Clear all
				</button>
			{/if}
		</div>

		<!-- League Filter -->
		<div class="mb-6">
			<button
				onclick={() => leagueExpanded = !leagueExpanded}
				class="flex items-center justify-between w-full text-left font-medium text-gray-900 mb-2"
			>
				<span>League</span>
				<svg
					class="w-5 h-5 transition-transform {leagueExpanded ? 'rotate-180' : ''}"
					fill="none"
					stroke="currentColor"
					viewBox="0 0 24 24"
				>
					<path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M19 9l-7 7-7-7" />
				</svg>
			</button>
			{#if leagueExpanded}
				{@render leagueTreeContent()}
			{/if}
		</div>

		<!-- Year Filter -->
		<div class="mb-6">
			<button
				onclick={() => yearExpanded = !yearExpanded}
				class="flex items-center justify-between w-full text-left font-medium text-gray-900 mb-2"
			>
				<span>Year</span>
				<svg
					class="w-5 h-5 transition-transform {yearExpanded ? 'rotate-180' : ''}"
					fill="none"
					stroke="currentColor"
					viewBox="0 0 24 24"
				>
					<path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M19 9l-7 7-7-7" />
				</svg>
			</button>
			{#if yearExpanded}
				<div class="space-y-2">
					{#each sortedYears as year}
						<label class="flex items-center space-x-2 cursor-pointer">
							<input
								type="checkbox"
								checked={selectedYears.includes(year)}
								onchange={() => toggleFilter('year', year.toString())}
								class="w-4 h-4 text-blue-600 border-gray-300 rounded focus:ring-blue-500"
							/>
							<span class="text-sm text-gray-700">{year}</span>
						</label>
					{/each}
				</div>
			{/if}
		</div>

		<!-- Team Filter -->
		<div class="mb-6">
			<button
				onclick={() => teamExpanded = !teamExpanded}
				class="flex items-center justify-between w-full text-left font-medium text-gray-900 mb-2"
			>
				<span>Team</span>
				<svg
					class="w-5 h-5 transition-transform {teamExpanded ? 'rotate-180' : ''}"
					fill="none"
					stroke="currentColor"
					viewBox="0 0 24 24"
				>
					<path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M19 9l-7 7-7-7" />
				</svg>
			</button>
			{#if teamExpanded}
				<div class="space-y-2">
					<input
						type="text"
						bind:value={teamSearchQuery}
						placeholder="Search teams..."
						class="w-full px-3 py-1.5 text-sm border border-gray-300 rounded focus:outline-none focus:ring-2 focus:ring-blue-500 focus:border-transparent mb-2"
					/>
					<div class="space-y-2 max-h-64 overflow-y-auto">
						{#each filteredTeams as team}
							<label class="flex items-center space-x-2 cursor-pointer">
								<input
									type="checkbox"
									checked={selectedTeams.includes(team.name)}
									onchange={() => toggleFilter('team', team.name)}
									class="w-4 h-4 text-blue-600 border-gray-300 rounded focus:ring-blue-500"
								/>
								<span class="text-sm text-gray-700">{team.name_pretty}</span>
							</label>
						{/each}
					</div>
				</div>
			{/if}
		</div>
	</div>
</aside>
