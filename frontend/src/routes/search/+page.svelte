<script lang="ts">
	import type { PageData } from './$types';
	import type { ScoredChunk } from '$lib/types';
	import PaperGroup from '$lib/components/PaperGroup.svelte';
	import { goto } from '$app/navigation';
	import { page } from '$app/stores';
	import { navigating } from '$app/stores';

	let { data }: { data: PageData } = $props();
	let isNavigating = $derived($navigating !== null);

	let selectedLeague = $state($page.url.searchParams.get('league') || '');
	let selectedYear = $state($page.url.searchParams.get('year') || '');
	let selectedTeam = $state($page.url.searchParams.get('team') || '');
	let mobileFiltersOpen = $state(false);

	interface PaperGroup {
		paperId: string;
		chunks: ScoredChunk[];
		avgScore: number;
	}

	const paperGroups = $derived(() => {
		if (!data.searchResult?.chunks) return [];

		const groups = new Map<string, ScoredChunk[]>();

		for (const scoredChunk of data.searchResult.chunks) {
			const paperId = scoredChunk.chunk.league_year_team_idx;
			if (!groups.has(paperId)) {
				groups.set(paperId, []);
			}
			groups.get(paperId)!.push(scoredChunk);
		}

		const groupArray: PaperGroup[] = [];
		for (const [paperId, chunks] of groups) {
			const avgScore = chunks.reduce((sum, c) => sum + c.score, 0) / chunks.length;
			groupArray.push({ paperId, chunks, avgScore });
		}

		groupArray.sort((a, b) => b.avgScore - a.avgScore);

		return groupArray;
	});

	function updateFilters() {
		const params = new URLSearchParams($page.url.searchParams);

		if (selectedLeague) {
			params.set('league', selectedLeague);
		} else {
			params.delete('league');
		}

		if (selectedYear) {
			params.set('year', selectedYear);
		} else {
			params.delete('year');
		}

		if (selectedTeam) {
			params.set('team', selectedTeam);
		} else {
			params.delete('team');
		}

		goto(`/search?${params.toString()}`);
	}

	function clearFilters() {
		selectedLeague = '';
		selectedYear = '';
		selectedTeam = '';
		const params = new URLSearchParams($page.url.searchParams);
		params.delete('league');
		params.delete('year');
		params.delete('team');
		goto(`/search?${params.toString()}`);
	}

	const hasFilters = $derived(selectedLeague || selectedYear || selectedTeam);
	const availableLeagues = $derived(data.searchResult?.suggestions.leagues || []);
	const availableTeams = $derived(data.searchResult?.suggestions.teams || []);

	function toggleMobileFilters() {
		mobileFiltersOpen = !mobileFiltersOpen;
	}
</script>

<div class="min-h-screen bg-gray-50">
	<div class="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8 py-6 sm:py-8">
		{#if data.error}
			<div class="bg-red-50 border border-red-200 rounded-lg p-4 mb-6">
				<p class="text-red-800">Error: {data.error}</p>
			</div>
		{/if}

		{#if !data.query}
			<div class="text-center py-12">
				<h1 class="text-xl sm:text-2xl font-semibold text-gray-900 mb-4">Search TDP Papers</h1>
				<p class="text-gray-600">Enter a search query to get started.</p>
			</div>
		{:else}
			<div class="mb-4 sm:mb-6">
				<h1 class="text-xl sm:text-2xl font-semibold text-gray-900 mb-2">
					Search Results for "{data.query}"
				</h1>
				{#if data.searchResult}
					<p class="text-sm sm:text-base text-gray-600">
						Found {data.searchResult.chunks.length} {data.searchResult.chunks.length === 1
							? 'result'
							: 'results'} in {paperGroups().length} {paperGroups().length === 1
							? 'paper'
							: 'papers'}
					</p>
				{/if}
			</div>

			<!-- Mobile Filter Button -->
			<button
				onclick={toggleMobileFilters}
				class="lg:hidden mb-4 w-full flex items-center justify-center gap-2 bg-white border border-gray-300 rounded-lg px-4 py-2 text-sm font-medium text-gray-700 hover:bg-gray-50"
			>
				<svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
					<path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 6V4m0 2a2 2 0 100 4m0-4a2 2 0 110 4m-6 8a2 2 0 100-4m0 4a2 2 0 110-4m0 4v2m0-6V4m6 6v10m6-2a2 2 0 100-4m0 4a2 2 0 110-4m0 4v2m0-6V4" />
				</svg>
				Filters {hasFilters ? '(active)' : ''}
			</button>

			<div class="flex flex-col lg:flex-row gap-6">
				<!-- Mobile Filter Overlay -->
				{#if mobileFiltersOpen}
					<button
						type="button"
						class="lg:hidden fixed inset-0 bg-black bg-opacity-50 z-40"
						onclick={toggleMobileFilters}
						aria-label="Close filters"
					></button>
				{/if}

				<!-- Sidebar Filters -->
				<aside class="lg:w-64 lg:flex-shrink-0 {mobileFiltersOpen ? 'fixed inset-y-0 left-0 z-50 w-80 bg-white shadow-xl overflow-y-auto' : 'hidden lg:block'}">
					<div class="bg-white border border-gray-200 rounded-lg p-4 lg:sticky lg:top-20">
						<div class="flex items-center justify-between mb-4">
							<h2 class="font-semibold text-gray-900">Filters</h2>
							<div class="flex items-center gap-2">
								{#if hasFilters}
									<button
										onclick={clearFilters}
										class="text-xs text-blue-600 hover:text-blue-800"
									>
										Clear
									</button>
								{/if}
								<button
									onclick={toggleMobileFilters}
									class="lg:hidden p-1 text-gray-500 hover:text-gray-700"
									aria-label="Close filters"
								>
									<svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
										<path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 18L18 6M6 6l12 12" />
									</svg>
								</button>
							</div>
						</div>

						<div class="space-y-4">
							<!-- League Filter -->
							<div>
								<label for="league" class="block text-sm font-medium text-gray-700 mb-1">
									League
								</label>
								<select
									id="league"
									bind:value={selectedLeague}
									onchange={updateFilters}
									class="w-full px-3 py-2 border border-gray-300 rounded-md text-sm focus:outline-none focus:ring-2 focus:ring-blue-500"
								>
									<option value="">All leagues</option>
									{#each availableLeagues as league}
										<option value={league}>{league}</option>
									{/each}
								</select>
							</div>

							<!-- Year Filter -->
							<div>
								<label for="year" class="block text-sm font-medium text-gray-700 mb-1">
									Year
								</label>
								<input
									id="year"
									type="text"
									bind:value={selectedYear}
									onchange={updateFilters}
									placeholder="e.g., 2023"
									class="w-full px-3 py-2 border border-gray-300 rounded-md text-sm focus:outline-none focus:ring-2 focus:ring-blue-500"
								/>
							</div>

							<!-- Team Filter -->
							<div>
								<label for="team" class="block text-sm font-medium text-gray-700 mb-1">
									Team
								</label>
								<select
									id="team"
									bind:value={selectedTeam}
									onchange={updateFilters}
									class="w-full px-3 py-2 border border-gray-300 rounded-md text-sm focus:outline-none focus:ring-2 focus:ring-blue-500"
								>
									<option value="">All teams</option>
									{#each availableTeams as team}
										<option value={team}>{team}</option>
									{/each}
								</select>
							</div>
						</div>
					</div>
				</aside>

				<!-- Main Results -->
				<main class="flex-1 min-w-0">
					{#if isNavigating}
						<div class="flex justify-center items-center py-16">
							<div class="text-center">
								<div class="animate-spin rounded-full h-12 w-12 border-b-2 border-blue-600 mx-auto mb-4"></div>
								<p class="text-gray-600">Loading results...</p>
							</div>
						</div>
					{:else if data.searchResult && data.searchResult.chunks.length > 0}
						<div class="space-y-4">
							{#each paperGroups() as group (group.paperId)}
								<PaperGroup
									paperId={group.paperId}
									chunks={group.chunks}
									query={data.query}
								/>
							{/each}
						</div>
					{:else if data.searchResult}
						<div class="bg-white border border-gray-200 rounded-lg p-6 sm:p-8 text-center">
							<svg class="mx-auto h-12 w-12 text-gray-400 mb-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
								<path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9.172 16.172a4 4 0 015.656 0M9 10h.01M15 10h.01M21 12a9 9 0 11-18 0 9 9 0 0118 0z" />
							</svg>
							<p class="text-gray-600 mb-2">No results found for "{data.query}"</p>
							{#if hasFilters}
								<p class="text-sm text-gray-500 mt-2">
									Try removing some filters to see more results.
								</p>
							{/if}
						</div>
					{/if}
				</main>
			</div>
		{/if}
	</div>
</div>
