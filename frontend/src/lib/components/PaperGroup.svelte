<script lang="ts">
	import type { SearchResultChunk } from '$lib/types';
	import ChunkResult from './ChunkResult.svelte';

	interface Props {
		paperId: string;
		chunks: SearchResultChunk[];
		query: string;
	}

	let { paperId, chunks, query }: Props = $props();

	let expanded = $state(false);

	const averageScore = $derived(
		chunks.reduce((sum, c) => sum + c.score, 0) / chunks.length
	);

	const firstChunk = $derived(chunks[0]);
	const topChunks = $derived(chunks.slice(0, 3));
	const remainingChunks = $derived(chunks.slice(3));
	const hasMore = $derived(remainingChunks.length > 0);

	function toggleExpanded() {
		expanded = !expanded;
	}
</script>

<div class="bg-white border border-gray-200 rounded-lg p-3 sm:p-4 shadow-sm hover:shadow-md transition-shadow">
	{#if firstChunk}
		<div class="mb-3">
			<div class="flex flex-col sm:flex-row sm:items-start sm:justify-between gap-2 sm:gap-4">
				<div class="flex-1 min-w-0">
					<a
						href="/paper/{paperId}"
						target="_blank"
						class="text-base sm:text-lg font-semibold text-blue-600 hover:text-blue-800 hover:underline break-words"
					>
						{firstChunk.team.name_pretty}
					</a>
					<div class="text-xs sm:text-sm text-gray-600 mt-1">
						{firstChunk.league.name_pretty} • {firstChunk.year}
					</div>
				</div>
				<div class="flex flex-row sm:flex-col items-center sm:items-end gap-2 sm:gap-0">
					<span class="text-xs text-gray-500 font-mono">
						avg: {averageScore.toFixed(3)}
					</span>
					<span class="text-xs text-gray-400">
						{chunks.length} {chunks.length === 1 ? 'match' : 'matches'}
					</span>
				</div>
			</div>
		</div>

		<div class="space-y-2 sm:space-y-3">
			{#each topChunks as chunk}
				<ChunkResult
					text={chunk.text}
					{query}
					score={chunk.score}
					breadcrumbs={chunk.breadcrumbs}
					title={chunk.title}
					{paperId}
				/>
			{/each}

			{#if hasMore}
				{#if expanded}
					{#each remainingChunks as chunk}
						<ChunkResult
							text={chunk.text}
							{query}
							score={chunk.score}
						/>
					{/each}
					<button
						onclick={toggleExpanded}
						class="text-xs sm:text-sm text-blue-600 hover:text-blue-800 font-medium"
					>
						Show less
					</button>
				{:else}
					<button
						onclick={toggleExpanded}
						class="text-xs sm:text-sm text-blue-600 hover:text-blue-800 font-medium"
					>
						Show {remainingChunks.length} more {remainingChunks.length === 1
							? 'match'
							: 'matches'}
					</button>
				{/if}
			{/if}
		</div>
	{/if}
</div>
