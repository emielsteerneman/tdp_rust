<script lang="ts">
	import { page } from '$app/stores';
	import SearchBar from './SearchBar.svelte';
	import ThemeToggle from './ThemeToggle.svelte';

	let currentQuery = $derived($page.url.searchParams.get('q') || '');
</script>

<nav class="sticky top-0 z-50 bg-white dark:bg-gray-900 border-b border-gray-200 dark:border-gray-800 shadow-sm">
	<div class="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8">
		<div class="flex items-center justify-between h-16">
			<!-- Logo/Title -->
			<div class="flex-shrink-0">
				<a href="/" class="flex items-center space-x-2">
					<span class="text-xl font-bold text-gray-900 dark:text-gray-100">TDP Browser</span>
				</a>
			</div>

			<!-- Search Bar (desktop) -->
			<div class="hidden md:flex flex-1 justify-center px-8">
				<SearchBar compact={true} initialValue={currentQuery} />
			</div>

			<!-- Theme Toggle -->
			<div class="flex-shrink-0">
				<ThemeToggle />
			</div>
		</div>

		<!-- Search Bar (mobile - always visible) -->
		<div class="md:hidden pb-3">
			<SearchBar compact={false} initialValue={currentQuery} />
		</div>
	</div>
</nav>
