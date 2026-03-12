<script lang="ts">
	import type { PageData } from './$types';
	import type { SearchResultChunk } from '$lib/types';
	import PaperGroup from '$lib/components/PaperGroup.svelte';
	import { page } from '$app/stores';
	import { navigating } from '$app/stores';

	let { data }: { data: PageData } = $props();
	let isNavigating = $derived($navigating !== null);

	interface PaperGroupData {
		paperId: string;
		chunks: SearchResultChunk[];
		avgScore: number;
	}

	const paperGroups = $derived.by(() => {
		if (!data.searchResult?.chunks) return [];

		const groups = new Map<string, SearchResultChunk[]>();

		for (const chunk of data.searchResult.chunks) {
			const paperId = chunk.league_year_team_idx;
			if (!groups.has(paperId)) {
				groups.set(paperId, []);
			}
			groups.get(paperId)!.push(chunk);
		}

		const groupArray: PaperGroupData[] = [];
		for (const [paperId, chunks] of groups) {
			const avgScore = chunks.reduce((sum, c) => sum + c.score, 0) / chunks.length;
			groupArray.push({ paperId, chunks, avgScore });
		}

		groupArray.sort((a, b) => b.avgScore - a.avgScore);

		return groupArray;
	});

	let hasFilters = $derived(
		$page.url.searchParams.has('league') ||
		$page.url.searchParams.has('year') ||
		$page.url.searchParams.has('team')
	);
</script>

<div class="max-w-7xl mx-auto px-4 py-6 sm:py-8">
	{#if data.error}
		<div class="bg-red-50 dark:bg-red-900/30 border border-red-200 dark:border-red-800 rounded-lg p-4 mb-6">
			<p class="text-red-800 dark:text-red-300">Error: {data.error}</p>
		</div>
	{/if}

	{#if !data.query}
		<div class="text-center py-12">
			<h1 class="text-xl sm:text-2xl font-semibold text-gray-900 dark:text-gray-100 mb-4">Search TDP Papers</h1>
			<p class="text-gray-600 dark:text-gray-400">Enter a search query to get started.</p>
		</div>
	{:else}
		<div class="mb-4 sm:mb-6">
			<h1 class="text-xl sm:text-2xl font-semibold text-gray-900 dark:text-gray-100 mb-2">
				Search Results for "{data.query}"
			</h1>
			{#if data.searchResult}
				<p class="text-sm sm:text-base text-gray-600 dark:text-gray-400">
					Found {data.searchResult.chunks.length} {data.searchResult.chunks.length === 1
						? 'result'
						: 'results'} in {paperGroups.length} {paperGroups.length === 1
						? 'paper'
						: 'papers'}
				</p>
			{/if}
		</div>

		{#if isNavigating}
			<div class="flex justify-center items-center py-16">
				<div class="text-center">
					<div class="animate-spin rounded-full h-12 w-12 border-b-2 border-blue-600 mx-auto mb-4"></div>
					<p class="text-gray-600 dark:text-gray-400">Loading results...</p>
				</div>
			</div>
		{:else if data.searchResult && data.searchResult.chunks.length > 0}
			<div class="space-y-4">
				{#each paperGroups as group (group.paperId)}
					<PaperGroup
						paperId={group.paperId}
						chunks={group.chunks}
						query={data.query}
					/>
				{/each}
			</div>
		{:else if data.searchResult}
			<div class="bg-white dark:bg-gray-900 border border-gray-200 dark:border-gray-800 rounded-lg p-6 sm:p-8 text-center">
				<svg class="mx-auto h-12 w-12 text-gray-400 dark:text-gray-500 mb-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
					<path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9.172 16.172a4 4 0 015.656 0M9 10h.01M15 10h.01M21 12a9 9 0 11-18 0 9 9 0 0118 0z" />
				</svg>
				<p class="text-gray-600 dark:text-gray-400 mb-2">No results found for "{data.query}"</p>
				{#if hasFilters}
					<p class="text-sm text-gray-500 dark:text-gray-400 mt-2">
						Try removing some filters to see more results.
					</p>
				{/if}
			</div>
		{/if}
	{/if}
</div>
