<script lang="ts">
	import FilterSidebar from '$lib/components/FilterSidebar.svelte';
	import ActiveFilterChips from '$lib/components/ActiveFilterChips.svelte';
	import type { LayoutData } from './$types';

	let { data, children }: { data: LayoutData; children: any } = $props();
</script>

<div class="flex flex-col lg:flex-row">
	<FilterSidebar
		leagues={data.leagues}
		years={data.years}
		teams={data.teams}
	/>

	<main class="flex-1 min-w-0">
		<ActiveFilterChips
			leagues={data.leagues}
			teams={data.teams}
		/>
		{@render children()}
	</main>
</div>
