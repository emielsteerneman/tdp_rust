<script lang="ts">
	import { getTeamInfo, updateTeamInfo, listTeams, type TeamMetadataEntry } from '$lib/api';
	import type { TeamName } from '$lib/types';

	let teams: TeamName[] = $state([]);
	let teamsLoading = $state(true);
	let searchQuery = $state('');
	let dropdownOpen = $state(false);
	let selectedTeam: TeamName | null = $state(null);

	let code = $state('');
	let entries: { key: string; value: string }[] = $state([]);
	let loaded = $state(false);
	let loading = $state(false);
	let saving = $state(false);
	let message = $state('');
	let error = $state('');

	let filteredTeams = $derived(
		searchQuery.trim()
			? teams.filter((t) =>
					t.name_pretty.toLowerCase().includes(searchQuery.toLowerCase())
				)
			: teams
	);

	// Load teams on mount
	$effect(() => {
		loadTeams();
	});

	async function loadTeams() {
		try {
			teams = await listTeams();
		} catch (e) {
			error = 'Failed to load teams list';
		} finally {
			teamsLoading = false;
		}
	}

	function selectTeam(team: TeamName) {
		selectedTeam = team;
		searchQuery = team.name_pretty;
		dropdownOpen = false;
		loaded = false;
		entries = [];
		message = '';
		error = '';
	}

	function handleSearchInput() {
		dropdownOpen = true;
		// If user edits away from the selected team, deselect
		if (selectedTeam && searchQuery !== selectedTeam.name_pretty) {
			selectedTeam = null;
			loaded = false;
		}
	}

	function handleSearchFocus() {
		dropdownOpen = true;
	}

	function handleSearchBlur() {
		// Delay to allow click on dropdown item
		setTimeout(() => {
			dropdownOpen = false;
		}, 200);
	}

	function handleSearchKeydown(event: KeyboardEvent) {
		if (event.key === 'Escape') {
			dropdownOpen = false;
		}
	}

	async function load() {
		if (!selectedTeam) return;
		loading = true;
		error = '';
		message = '';
		try {
			const result = await getTeamInfo(selectedTeam.name_pretty);
			entries = result.map((e) => ({ key: e.key, value: e.value }));
			loaded = true;
		} catch (e) {
			error = e instanceof Error ? e.message : 'Failed to load';
		} finally {
			loading = false;
		}
	}

	function addEntry() {
		entries = [...entries, { key: '', value: '' }];
	}

	function removeEntry(index: number) {
		entries = entries.filter((_, i) => i !== index);
	}

	async function save() {
		if (!selectedTeam) return;
		if (!code.trim()) {
			error = 'Please enter your team code';
			return;
		}
		saving = true;
		error = '';
		message = '';
		try {
			const result = await updateTeamInfo(selectedTeam.name, code, entries);
			message = result;
		} catch (e) {
			error = e instanceof Error ? e.message : 'Failed to save';
		} finally {
			saving = false;
		}
	}
</script>

<svelte:head>
	<title>Edit Team Info</title>
</svelte:head>

<div class="min-h-screen bg-gray-50 dark:bg-gray-950 py-8 px-4">
	<div class="max-w-2xl mx-auto">
		<!-- Header -->
		<div class="mb-8">
			<h1 class="text-2xl font-bold text-gray-900 dark:text-gray-100">Edit Team Info</h1>
			<p class="mt-1 text-sm text-gray-500 dark:text-gray-400">
				Update your team's metadata — websites, repositories, social links.
			</p>
		</div>

		<!-- Team selection + code + load -->
		<div class="bg-white dark:bg-gray-900 rounded-lg border border-gray-200 dark:border-gray-800 p-6 shadow-sm">
			<div class="space-y-4">
				<!-- Team search dropdown -->
				<div>
					<label for="team-search" class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1">
						Team
					</label>
					<div class="relative">
						<input
							id="team-search"
							type="text"
							bind:value={searchQuery}
							oninput={handleSearchInput}
							onfocus={handleSearchFocus}
							onblur={handleSearchBlur}
							onkeydown={handleSearchKeydown}
							placeholder={teamsLoading ? 'Loading teams...' : 'Search for your team...'}
							disabled={teamsLoading}
							autocomplete="off"
							class="w-full px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-lg
								bg-white dark:bg-gray-800 text-gray-900 dark:text-gray-100
								placeholder-gray-400 dark:placeholder-gray-500
								focus:outline-none focus:ring-2 focus:ring-blue-500 focus:border-transparent
								disabled:opacity-50 disabled:cursor-not-allowed"
						/>
						<!-- Dropdown chevron -->
						<svg
							class="absolute right-3 top-1/2 -translate-y-1/2 w-4 h-4 text-gray-400 pointer-events-none"
							fill="none" stroke="currentColor" viewBox="0 0 24 24"
						>
							<path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M19 9l-7 7-7-7" />
						</svg>

						<!-- Dropdown list -->
						{#if dropdownOpen && !teamsLoading && filteredTeams.length > 0}
							<ul class="absolute z-50 mt-1 w-full max-h-60 overflow-auto rounded-lg border border-gray-200 dark:border-gray-700 bg-white dark:bg-gray-800 shadow-lg">
								{#each filteredTeams as team}
									<li>
										<button
											type="button"
											class="w-full text-left px-3 py-2 text-sm text-gray-900 dark:text-gray-100
												hover:bg-blue-50 dark:hover:bg-gray-700 cursor-pointer
												{selectedTeam?.name === team.name ? 'bg-blue-50 dark:bg-gray-700 font-medium' : ''}"
											onmousedown={() => selectTeam(team)}
										>
											{team.name_pretty}
										</button>
									</li>
								{/each}
							</ul>
						{/if}
						{#if dropdownOpen && !teamsLoading && searchQuery.trim() && filteredTeams.length === 0}
							<div class="absolute z-50 mt-1 w-full rounded-lg border border-gray-200 dark:border-gray-700 bg-white dark:bg-gray-800 shadow-lg px-3 py-2 text-sm text-gray-500 dark:text-gray-400">
								No teams found
							</div>
						{/if}
					</div>
				</div>

				<!-- Team code -->
				<div>
					<label for="team-code" class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1">
						Team Code
					</label>
					<input
						id="team-code"
						type="password"
						bind:value={code}
						placeholder="Your team code or master password"
						class="w-full px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-lg
							bg-white dark:bg-gray-800 text-gray-900 dark:text-gray-100
							placeholder-gray-400 dark:placeholder-gray-500
							focus:outline-none focus:ring-2 focus:ring-blue-500 focus:border-transparent"
					/>
				</div>

				<!-- Load button -->
				<button
					onclick={load}
					disabled={loading || !selectedTeam}
					class="w-full px-4 py-2 rounded-lg font-medium text-white
						bg-blue-600 hover:bg-blue-700 disabled:bg-gray-300 dark:disabled:bg-gray-700
						disabled:cursor-not-allowed transition-colors"
				>
					{loading ? 'Loading...' : 'Load current entries'}
				</button>
			</div>
		</div>

		<!-- Metadata entries -->
		{#if loaded}
			<div class="mt-6 bg-white dark:bg-gray-900 rounded-lg border border-gray-200 dark:border-gray-800 p-6 shadow-sm">
				<div class="flex items-center justify-between mb-4">
					<h2 class="text-lg font-semibold text-gray-900 dark:text-gray-100">Metadata</h2>
					<span class="text-xs text-gray-400 dark:text-gray-500">
						{entries.length} {entries.length === 1 ? 'entry' : 'entries'}
					</span>
				</div>

				{#if entries.length === 0}
					<p class="text-sm text-gray-500 dark:text-gray-400 mb-4">
						No entries yet. Add your team's websites, repos, and links below.
					</p>
				{/if}

				<div class="space-y-4">
					{#each entries as entry, i}
						<div class="relative rounded-lg border border-gray-200 dark:border-gray-700 p-3">
							<button
								onclick={() => removeEntry(i)}
								class="absolute top-2 right-2 p-1 rounded text-gray-400 hover:text-red-500 hover:bg-red-50 dark:hover:bg-red-900/20 transition-colors"
								title="Remove entry"
							>
								<svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
									<path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 18L18 6M6 6l12 12" />
								</svg>
							</button>
							<div class="space-y-2 pr-8">
								<input
									type="text"
									bind:value={entry.key}
									placeholder="key (e.g. github)"
									class="w-full px-3 py-2 text-sm border border-gray-300 dark:border-gray-600 rounded-lg
										bg-white dark:bg-gray-800 text-gray-900 dark:text-gray-100
										placeholder-gray-400 dark:placeholder-gray-500
										focus:outline-none focus:ring-2 focus:ring-blue-500 focus:border-transparent"
								/>
								<input
									type="text"
									bind:value={entry.value}
									placeholder="value (e.g. https://github.com/...)"
									class="w-full px-3 py-2 text-sm border border-gray-300 dark:border-gray-600 rounded-lg
										bg-white dark:bg-gray-800 text-gray-900 dark:text-gray-100
										placeholder-gray-400 dark:placeholder-gray-500
										focus:outline-none focus:ring-2 focus:ring-blue-500 focus:border-transparent"
								/>
							</div>
						</div>
					{/each}
				</div>

				<div class="mt-4 flex gap-3">
					<button
						onclick={addEntry}
						class="px-4 py-2 text-sm rounded-lg border border-gray-300 dark:border-gray-600
							text-gray-700 dark:text-gray-300 hover:bg-gray-50 dark:hover:bg-gray-800 transition-colors"
					>
						+ Add entry
					</button>
					<button
						onclick={save}
						disabled={saving}
						class="px-4 py-2 text-sm rounded-lg font-medium text-white
							bg-blue-600 hover:bg-blue-700 disabled:bg-gray-300 dark:disabled:bg-gray-700
							disabled:cursor-not-allowed transition-colors"
					>
						{saving ? 'Saving...' : 'Save changes'}
					</button>
				</div>
			</div>
		{/if}

		<!-- Messages -->
		{#if message}
			<div class="mt-4 px-4 py-3 rounded-lg bg-green-50 dark:bg-green-900/20 border border-green-200 dark:border-green-800 text-sm text-green-700 dark:text-green-400">
				{message}
			</div>
		{/if}
		{#if error}
			<div class="mt-4 px-4 py-3 rounded-lg bg-red-50 dark:bg-red-900/20 border border-red-200 dark:border-red-800 text-sm text-red-700 dark:text-red-400">
				{error}
			</div>
		{/if}
	</div>
</div>
