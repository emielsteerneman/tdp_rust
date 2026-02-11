<script lang="ts">
	import type { TDPName } from '$lib/types';

	interface Props {
		paper: TDPName;
	}

	let { paper }: Props = $props();

	let paperId = $derived(
		`${paper.league.name}__${paper.year}__${paper.team_name.name}__${paper.index}`
	);
</script>

<a
	href="/paper/{paperId}"
	class="inline-block px-3 py-1 text-sm rounded-lg border border-gray-200 bg-white text-gray-800 hover:border-blue-300 hover:bg-blue-50 transition-colors"
>
	{paper.team_name.name_pretty}{#if paper.index > 0}<span class="text-gray-400 ml-0.5">#{paper.index + 1}</span>{/if}
</a>
