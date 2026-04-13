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

			<!-- Right side: Links + Theme Toggle -->
			<div class="flex-shrink-0 flex items-center space-x-2 sm:space-x-4">
				<a href="/connect/mcp" title="Connect your AI / scripts" class="text-sm px-2 sm:px-3 py-1.5 rounded-md border border-gray-300 dark:border-gray-600 text-gray-700 dark:text-gray-300 hover:bg-gray-100 dark:hover:bg-gray-800 transition-colors">
					<!-- Icon (mobile) -->
					<svg class="w-4 h-4 sm:hidden" fill="none" stroke="currentColor" viewBox="0 0 24 24">
						<path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M13.828 10.172a4 4 0 00-5.656 0l-4 4a4 4 0 105.656 5.656l1.102-1.101m-.758-4.899a4 4 0 005.656 0l4-4a4 4 0 00-5.656-5.656l-1.1 1.1" />
					</svg>
					<!-- Text (desktop) -->
					<span class="hidden sm:inline">Connect your AI / scripts</span>
				</a>
				<a href="/teams/edit" title="Edit team info" class="text-sm px-2 sm:px-3 py-1.5 rounded-md border border-gray-300 dark:border-gray-600 text-gray-700 dark:text-gray-300 hover:bg-gray-100 dark:hover:bg-gray-800 transition-colors">
					<!-- Icon (mobile) -->
					<svg class="w-4 h-4 sm:hidden" fill="none" stroke="currentColor" viewBox="0 0 24 24">
						<path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M11 5H6a2 2 0 00-2 2v11a2 2 0 002 2h11a2 2 0 002-2v-5m-1.414-9.414a2 2 0 112.828 2.828L11.828 15H9v-2.828l8.586-8.586z" />
					</svg>
					<!-- Text (desktop) -->
					<span class="hidden sm:inline">Edit team info</span>
				</a>
				<a href="/suggestions" title="Leave a suggestion" class="text-sm px-2 sm:px-3 py-1.5 rounded-md border border-gray-300 dark:border-gray-600 text-gray-700 dark:text-gray-300 hover:bg-gray-100 dark:hover:bg-gray-800 transition-colors">
					<!-- Icon (mobile) -->
					<svg class="w-4 h-4 sm:hidden" fill="none" stroke="currentColor" viewBox="0 0 24 24">
						<path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M8 10h.01M12 10h.01M16 10h.01M21 12c0 4.418-4.03 8-9 8a9.863 9.863 0 01-4.255-.949L3 20l1.395-3.72C3.512 15.042 3 13.574 3 12c0-4.418 4.03-8 9-8s9 3.582 9 8z" />
					</svg>
					<!-- Text (desktop) -->
					<span class="hidden sm:inline">Leave a suggestion</span>
				</a>
				<ThemeToggle />
			</div>
		</div>

		<!-- Search Bar (mobile - always visible) -->
		<div class="md:hidden pb-3">
			<SearchBar compact={false} initialValue={currentQuery} />
		</div>
	</div>
</nav>
