<script lang="ts">
	import type { BreadcrumbEntry } from '$lib/types';
	import { slugifyHeading } from '$lib/markdown';

	interface Props {
		text: string;
		query: string;
		score: number;
		breadcrumbs?: BreadcrumbEntry[];
		title?: string;
		paperId: string;
	}

	let { text, query, score, breadcrumbs, title, paperId }: Props = $props();

	function highlightText(text: string, query: string): string {
		if (!query.trim()) return text;

		const words = query
			.trim()
			.split(/\s+/)
			.filter((w) => w.length > 0);

		let result = text;
		for (const word of words) {
			const regex = new RegExp(`(${escapeRegex(word)})`, 'gi');
			result = result.replace(regex, '<mark class="bg-yellow-200">$1</mark>');
		}

		return result;
	}

	function escapeRegex(str: string): string {
		return str.replace(/[.*+?^${}()|[\]\\]/g, '\\$&');
	}

	const highlighted = $derived(highlightText(text, query));

	// Build full breadcrumb trail: ancestors + chunk's own title
	const crumbs = $derived.by(() => {
		const trail: { title: string; href: string }[] = [];
		for (const b of breadcrumbs ?? []) {
			trail.push({
				title: b.title,
				href: `/paper/${paperId}#${slugifyHeading(b.title)}`
			});
		}
		if (title) {
			trail.push({
				title,
				href: `/paper/${paperId}#${slugifyHeading(title)}`
			});
		}
		return trail;
	});
</script>

<div class="text-xs sm:text-sm text-gray-700 leading-relaxed">
	{#if crumbs.length > 0}
		<div class="text-xs text-gray-400 mb-1">
			{#each crumbs as crumb, i}
				{#if i > 0}
					<span class="mx-1">&gt;</span>
				{/if}
				<a href={crumb.href} class="hover:text-blue-600 hover:underline">{crumb.title}</a>
			{/each}
		</div>
	{/if}
	<div class="flex items-start justify-between gap-2 mb-1">
		<div class="flex-1 min-w-0 break-words">
			{@html highlighted}
		</div>
		<span class="text-xs text-gray-500 font-mono flex-shrink-0">
			{score.toFixed(3)}
		</span>
	</div>
</div>
