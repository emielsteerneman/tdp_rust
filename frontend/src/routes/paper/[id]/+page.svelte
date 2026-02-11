<script lang="ts">
	import { marked } from 'marked';
	import type { PageData } from './$types';
	import { navigating } from '$app/stores';

	let { data }: { data: PageData } = $props();
	let isNavigating = $derived($navigating !== null);

	// Configure marked for safe HTML rendering
	marked.setOptions({
		gfm: true,
		breaks: false
	});

	// Render markdown to HTML
	const htmlContent = $derived(marked.parse(data.content) as string);

	// Format team name for display (replace underscores with spaces)
	const teamDisplay = $derived(data.metadata.team.replace(/_/g, ' '));
	const leagueDisplay = $derived(data.metadata.league.replace(/_/g, ' '));
</script>

<div class="min-h-screen bg-gray-50">
	<div class="max-w-4xl mx-auto px-4 py-6 sm:py-8">
		{#if isNavigating}
			<div class="flex justify-center items-center py-16">
				<div class="text-center">
					<div class="animate-spin rounded-full h-12 w-12 border-b-2 border-blue-600 mx-auto mb-4"></div>
					<p class="text-gray-600">Loading paper...</p>
				</div>
			</div>
		{:else}
		<!-- Metadata Bar -->
		<div class="bg-white rounded-lg shadow-sm border border-gray-200 p-4 sm:p-6 mb-4 sm:mb-6">
			<div class="flex flex-col sm:flex-row sm:flex-wrap sm:items-center gap-2 sm:gap-4 text-sm">
				<div class="flex items-center gap-2">
					<span class="font-semibold text-gray-700">Team:</span>
					<span class="text-gray-900">{teamDisplay}</span>
				</div>
				<div class="hidden sm:block h-4 w-px bg-gray-300"></div>
				<div class="flex items-center gap-2">
					<span class="font-semibold text-gray-700">League:</span>
					<span class="text-gray-900">{leagueDisplay}</span>
				</div>
				<div class="hidden sm:block h-4 w-px bg-gray-300"></div>
				<div class="flex items-center gap-2">
					<span class="font-semibold text-gray-700">Year:</span>
					<span class="text-gray-900">{data.metadata.year}</span>
				</div>
			</div>
		</div>

		<!-- Paper Content -->
		<div class="bg-white rounded-lg shadow-sm border border-gray-200 p-4 sm:p-6 md:p-8 mb-4 sm:mb-6">
			<article
				class="prose prose-gray prose-sm sm:prose-base max-w-none prose-headings:font-bold prose-h1:text-2xl sm:prose-h1:text-3xl prose-h2:text-xl sm:prose-h2:text-2xl prose-h3:text-lg sm:prose-h3:text-xl prose-p:leading-relaxed prose-a:text-blue-600 hover:prose-a:text-blue-800"
			>
				{@html htmlContent}
			</article>
		</div>

		<!-- Actions -->
		<div class="flex justify-center">
			<button
				disabled
				class="relative px-4 sm:px-6 py-2 sm:py-3 bg-gray-300 text-gray-500 rounded-lg font-medium cursor-not-allowed group text-sm sm:text-base"
				title="Coming soon"
			>
				View Original PDF
				<span
					class="absolute bottom-full left-1/2 transform -translate-x-1/2 mb-2 px-3 py-1 bg-gray-900 text-white text-xs rounded opacity-0 group-hover:opacity-100 transition-opacity whitespace-nowrap pointer-events-none"
				>
					Coming soon
				</span>
			</button>
		</div>
		{/if}
	</div>
</div>
