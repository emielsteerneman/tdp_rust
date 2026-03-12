<script lang="ts">
	import { goto } from '$app/navigation';
	import { page } from '$app/stores';

	interface Props {
		initialValue?: string;
		compact?: boolean;
	}

	let { initialValue = '', compact = false }: Props = $props();

	let query = $state('');

	$effect(() => {
		query = initialValue;
	});

	function handleSearch() {
		if (query.trim()) {
			const params = new URLSearchParams();
			params.set('q', query.trim());
			// Preserve filter params from current URL
			for (const key of ['league', 'year', 'team']) {
				for (const value of $page.url.searchParams.getAll(key)) {
					params.append(key, value);
				}
			}
			goto(`/search?${params.toString()}`);
		}
	}

	function handleKeydown(event: KeyboardEvent) {
		if (event.key === 'Enter') {
			handleSearch();
		}
	}
</script>

<div class="relative w-full {compact ? 'max-w-md' : 'max-w-2xl'}">
	<div class="relative">
		<input
			type="text"
			bind:value={query}
			onkeydown={handleKeydown}
			placeholder="Search TDP papers..."
			class="w-full px-4 py-2 pl-10 pr-4 border border-gray-300 dark:border-gray-600 rounded-lg focus:outline-none focus:ring-2 focus:ring-blue-500 focus:border-transparent bg-white dark:bg-gray-800 text-gray-900 dark:text-gray-100 placeholder-gray-400 dark:placeholder-gray-500"
		/>
		<svg
			class="absolute left-3 top-1/2 -translate-y-1/2 w-5 h-5 text-gray-400 dark:text-gray-500"
			fill="none"
			stroke="currentColor"
			viewBox="0 0 24 24"
		>
			<path
				stroke-linecap="round"
				stroke-linejoin="round"
				stroke-width="2"
				d="M21 21l-6-6m2-5a7 7 0 11-14 0 7 7 0 0114 0z"
			/>
		</svg>
	</div>
</div>
