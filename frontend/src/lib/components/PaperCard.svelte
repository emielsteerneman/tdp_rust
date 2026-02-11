<script lang="ts">
	import type { TDPName } from '$lib/types';

	interface Props {
		paper: TDPName;
	}

	let { paper }: Props = $props();

	// Construct the paper ID for the link
	let paperId = $derived(
		`${paper.league.name}__${paper.year}__${paper.team_name.name}__${paper.index}`
	);

	// League badge color based on league type
	function getLeagueBadgeColor(leagueName: string): string {
		if (leagueName.includes('smallsize')) return 'bg-blue-100 text-blue-800';
		if (leagueName.includes('middlesize')) return 'bg-green-100 text-green-800';
		if (leagueName.includes('humanoid')) return 'bg-purple-100 text-purple-800';
		if (leagueName.includes('standard_platform')) return 'bg-orange-100 text-orange-800';
		if (leagueName.includes('rescue')) return 'bg-red-100 text-red-800';
		if (leagueName.includes('athome')) return 'bg-yellow-100 text-yellow-800';
		if (leagueName.includes('industrial')) return 'bg-gray-100 text-gray-800';
		return 'bg-gray-100 text-gray-800';
	}
</script>

<a
	href="/paper/{paperId}"
	class="block bg-white border border-gray-200 rounded-lg p-3 sm:p-4 hover:shadow-md hover:border-blue-300 transition-all"
>
	<div class="flex flex-col sm:flex-row sm:items-start sm:justify-between gap-2 mb-2">
		<h3 class="font-bold text-gray-900 text-base sm:text-lg flex-1">
			{paper.team_name.name_pretty}
		</h3>
		<span
			class="self-start px-2 py-1 text-xs font-medium rounded {getLeagueBadgeColor(paper.league.name)} whitespace-nowrap"
		>
			{paper.league.name_pretty}
		</span>
	</div>
	<div class="text-sm text-gray-600">
		{paper.year}
		{#if paper.index > 0}
			<span class="text-gray-400">• #{paper.index + 1}</span>
		{/if}
	</div>
</a>
