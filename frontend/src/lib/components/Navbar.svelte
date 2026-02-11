<script lang="ts">
	import SearchBar from './SearchBar.svelte';

	let mobileMenuOpen = $state(false);
	let mobileSearchOpen = $state(false);

	function toggleMobileMenu() {
		mobileMenuOpen = !mobileMenuOpen;
		if (mobileMenuOpen) mobileSearchOpen = false;
	}

	function toggleMobileSearch() {
		mobileSearchOpen = !mobileSearchOpen;
		if (mobileSearchOpen) mobileMenuOpen = false;
	}
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

			<!-- Mobile Actions -->
			<div class="flex md:hidden items-center space-x-2">
				<!-- Mobile Search Button -->
				<button
					onclick={toggleMobileSearch}
					class="p-2 text-gray-700 hover:text-gray-900"
					aria-label="Toggle search"
				>
					<svg class="w-6 h-6" fill="none" stroke="currentColor" viewBox="0 0 24 24">
						<path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M21 21l-6-6m2-5a7 7 0 11-14 0 7 7 0 0114 0z" />
					</svg>
				</button>

				<!-- Mobile Menu Button -->
				<button
					onclick={toggleMobileMenu}
					class="p-2 text-gray-700 hover:text-gray-900"
					aria-label="Toggle menu"
				>
					{#if mobileMenuOpen}
						<svg class="w-6 h-6" fill="none" stroke="currentColor" viewBox="0 0 24 24">
							<path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 18L18 6M6 6l12 12" />
						</svg>
					{:else}
						<svg class="w-6 h-6" fill="none" stroke="currentColor" viewBox="0 0 24 24">
							<path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M4 6h16M4 12h16M4 18h16" />
						</svg>
					{/if}
				</button>
			</div>

			<!-- Navigation Links (desktop) -->
			<div class="hidden md:flex items-center space-x-6">
				<a
					href="/"
					class="text-gray-700 hover:text-gray-900 font-medium transition-colors"
				>
					Home
				</a>
			</div>
		</div>

		<!-- Mobile Search Bar -->
		{#if mobileSearchOpen}
			<div class="md:hidden pb-4">
				<SearchBar compact={false} />
			</div>
		{/if}

		<!-- Mobile Menu -->
		{#if mobileMenuOpen}
			<div class="md:hidden py-4 space-y-2 border-t border-gray-200">
				<a
					href="/"
					class="block px-4 py-2 text-gray-700 hover:bg-gray-50 hover:text-gray-900 font-medium transition-colors"
					onclick={() => mobileMenuOpen = false}
				>
					Home
				</a>
			</div>
		{/if}
	</div>
</nav>
