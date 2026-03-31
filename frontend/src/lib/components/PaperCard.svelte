<script lang="ts">
	import type { TDPName } from '$lib/types';

	interface Props {
		paper: TDPName;
	}

	let { paper }: Props = $props();

	let paperId = $derived(`${paper.league.name}__${paper.year}__${paper.team_name.name}`);
</script>

<a
	href="/paper/{paperId}"
	target="_blank"
	class="inline-block px-3 py-1 text-sm rounded-lg border border-gray-200 dark:border-gray-700 bg-white dark:bg-gray-800 text-gray-800 dark:text-gray-200 hover:border-blue-300 dark:hover:border-blue-600 hover:bg-blue-50 dark:hover:bg-blue-900/30 transition-colors"
>
	{paper.team_name.name_pretty}
</a>
