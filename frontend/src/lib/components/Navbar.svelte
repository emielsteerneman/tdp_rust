<script lang="ts">
	import SearchBar from './SearchBar.svelte';

	let mobileSearchOpen = $state(false);
</script>

<nav class="sticky top-0 z-50 bg-white border-b border-gray-200 shadow-sm">
	<div class="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8">
		<div class="flex items-center justify-between h-16">
			<!-- Logo/Title -->
			<div class="flex-shrink-0">
				<a href="/" class="flex items-center space-x-2">
					<span class="text-xl font-bold text-gray-900">TDP Browser</span>
				</a>
			</div>

			<!-- Search Bar (desktop - center) -->
			<div class="hidden md:flex flex-1 justify-center px-8">
				<SearchBar compact={true} />
			</div>

			<!-- Mobile Search Button -->
			<button
				class="md:hidden p-2 text-gray-700 hover:text-gray-900"
				aria-label="Toggle search"
				onclick={() => mobileSearchOpen = !mobileSearchOpen}
			>
				<svg class="w-6 h-6" fill="none" stroke="currentColor" viewBox="0 0 24 24">
					<path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M21 21l-6-6m2-5a7 7 0 11-14 0 7 7 0 0114 0z" />
				</svg>
			</button>
		</div>

		<!-- Mobile Search Bar -->
		{#if mobileSearchOpen}
			<div class="md:hidden pb-4">
				<SearchBar compact={false} />
			</div>
		{/if}
	</div>
</nav>
