<script lang="ts">
	import type { TocHeading } from '$lib/markdown';

	interface Props {
		headings: TocHeading[];
		activeId: string;
	}

	let { headings, activeId }: Props = $props();

	function scrollToHeading(id: string) {
		const el = document.getElementById(id);
		if (el) {
			el.scrollIntoView({ behavior: 'smooth', block: 'start' });
		}
	}
</script>

<aside class="hidden lg:block w-64 flex-shrink-0 bg-white dark:bg-gray-900 border-r border-gray-200 dark:border-gray-800 h-screen sticky top-16 overflow-y-auto">
	<div class="p-4">
		<h2 class="text-sm font-semibold text-gray-500 dark:text-gray-400 uppercase tracking-wider mb-3">Contents</h2>
		<nav class="space-y-1">
			{#each headings as heading}
				<button
					onclick={() => scrollToHeading(heading.id)}
					class="block w-full text-left text-sm leading-snug py-1 rounded transition-colors
						{heading.level === 3 ? 'pl-3' : heading.level >= 4 ? 'pl-6' : ''}
						{activeId === heading.id
							? 'text-blue-600 dark:text-blue-400 font-medium'
							: 'text-gray-500 dark:text-gray-400 hover:text-gray-800 dark:hover:text-gray-200'}"
				>
					{heading.text}
				</button>
			{/each}
		</nav>
	</div>
</aside>
