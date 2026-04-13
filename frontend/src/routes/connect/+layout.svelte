<script lang="ts">
	import { page } from '$app/stores';

	const tabs = [
		{ label: 'Your AI', href: '/connect/mcp' },
		{ label: 'Your scripts', href: '/connect/api' },
	];

	let { children } = $props();
</script>

<div class="min-h-screen bg-gray-50 dark:bg-gray-950 py-8 px-4">
	<div class="max-w-2xl mx-auto">
		<!-- Tab navigation -->
		<div class="flex space-x-1 mb-6 bg-gray-200 dark:bg-gray-800 rounded-lg p-1 max-w-xs">
			{#each tabs as tab}
				{#if $page.url.pathname === tab.href}
					<span class="flex-1 text-center px-3 py-1.5 text-sm rounded-md bg-white dark:bg-gray-700 text-gray-900 dark:text-gray-100 font-medium shadow-sm">
						{tab.label}
					</span>
				{:else}
					<a href={tab.href} class="flex-1 text-center px-3 py-1.5 text-sm rounded-md text-gray-600 dark:text-gray-400 hover:text-gray-900 dark:hover:text-gray-200 transition-colors">
						{tab.label}
					</a>
				{/if}
			{/each}
		</div>

		{@render children()}
	</div>
</div>
