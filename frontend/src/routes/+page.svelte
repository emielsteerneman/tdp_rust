<script lang="ts">
	import { page } from '$app/stores';
	import SearchBar from '$lib/components/SearchBar.svelte';
	import FilterSidebar from '$lib/components/FilterSidebar.svelte';
	import PaperCard from '$lib/components/PaperCard.svelte';
	import type { TDPName, League, TeamName } from '$lib/types';

	interface Props {
		data: {
			papers: TDPName[];
			teams: TeamName[];
			leagues: League[];
			years: number[];
		};
	}

	let { data }: Props = $props();

	// Get selected filters from URL
	let selectedLeagues = $derived($page.url.searchParams.getAll('league'));
	let selectedYears = $derived($page.url.searchParams.getAll('year').map(y => parseInt(y)));
	let selectedTeams = $derived($page.url.searchParams.getAll('team'));

	// Filter papers based on selected filters
	let filteredPapers = $derived(
		data.papers.filter((paper) => {
			if (selectedLeagues.length > 0 && !selectedLeagues.includes(paper.league.name)) {
				return false;
			}

			if (selectedYears.length > 0 && !selectedYears.includes(paper.year)) {
				return false;
			}

			if (selectedTeams.length > 0 && !selectedTeams.includes(paper.team_name.name)) {
				return false;
			}

			return true;
		})
	);

	let hasActiveFilters = $derived(
		selectedLeagues.length > 0 || selectedYears.length > 0 || selectedTeams.length > 0
	);

	let isLoading = $state(false);
</script>

<div class="flex flex-col lg:flex-row">
	<!-- Filter Sidebar -->
	<FilterSidebar
		leagues={data.leagues}
		years={data.years}
		teams={data.teams}
	/>

	<!-- Main Content -->
	<main class="flex-1 min-w-0">
		<!-- Hero Section -->
		<div class="bg-gradient-to-b from-blue-50 to-white py-8 sm:py-12 px-4">
			<div class="max-w-4xl mx-auto text-center">
				<h1 class="text-3xl sm:text-4xl md:text-5xl font-bold text-gray-900 mb-4">
					RoboCup TDP Browser
				</h1>
				<p class="text-base sm:text-lg text-gray-600 mb-6 sm:mb-8">
					Explore Team Description Papers from RoboCup competitions
				</p>
				<div class="flex justify-center">
					<SearchBar />
				</div>
			</div>
		</div>

		<!-- Papers Grid -->
		<div class="max-w-7xl mx-auto px-4 py-6 sm:py-8">
			<div class="mb-6">
				<h2 class="text-xl sm:text-2xl font-semibold text-gray-900">
					{#if hasActiveFilters}
						Filtered Papers
					{:else}
						All Papers
					{/if}
				</h2>
				<p class="text-sm text-gray-600 mt-1">
					Showing {filteredPapers.length} of {data.papers.length} papers
				</p>
			</div>

			{#if isLoading}
				<div class="flex justify-center items-center py-16">
					<div class="animate-spin rounded-full h-12 w-12 border-b-2 border-blue-600"></div>
				</div>
			{:else if filteredPapers.length === 0}
				<div class="text-center py-12 bg-gray-50 rounded-lg">
					<svg class="mx-auto h-12 w-12 text-gray-400 mb-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
						<path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 12h6m-6 4h6m2 5H7a2 2 0 01-2-2V5a2 2 0 012-2h5.586a1 1 0 01.707.293l5.414 5.414a1 1 0 01.293.707V19a2 2 0 01-2 2z" />
					</svg>
					<p class="text-gray-500 text-lg mb-2">
						No papers match the selected filters.
					</p>
					{#if hasActiveFilters}
						<p class="text-sm text-gray-400">
							Try adjusting your filters to see more results.
						</p>
					{/if}
				</div>
			{:else}
				<div class="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-3 xl:grid-cols-4 gap-4">
					{#each filteredPapers as paper}
						<PaperCard {paper} />
					{/each}
				</div>
			{/if}
		</div>
	</main>
</div>
