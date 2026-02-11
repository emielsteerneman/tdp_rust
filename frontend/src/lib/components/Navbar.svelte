<script lang="ts">
	import SearchBar from './SearchBar.svelte';
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

			<!-- Search Bar (desktop) -->
			<div class="hidden md:flex flex-1 justify-center px-8">
				<SearchBar compact={true} />
			</div>
		</div>

		<!-- Search Bar (mobile - always visible) -->
		<div class="md:hidden pb-3">
			<SearchBar compact={false} />
		</div>
	</div>
</nav>
