<script lang="ts">
	import type { BreadcrumbEntry } from '$lib/types';
	import { slugifyHeading } from '$lib/markdown';

	interface Props {
		text: string;
		highlightTerms: string[];
		score: number;
		breadcrumbs?: BreadcrumbEntry[];
		title?: string;
		paperId: string;
	}

	let { text, highlightTerms, score, breadcrumbs, title, paperId }: Props = $props();

	function highlightText(text: string, terms: string[]): string {
		if (terms.length === 0) return text;

		// Sort by length descending so longer terms (bigrams/trigrams) match
		// before their constituent unigrams in the regex alternation
		const sorted = [...terms].sort((a, b) => b.length - a.length);

		// Phase 1: collect all match ranges against the original plain text
		const pattern = sorted.map(escapeRegex).join('|');
		const regex = new RegExp(pattern, 'gi');
		const matches: { start: number; end: number }[] = [];
		let m: RegExpExecArray | null;
		while ((m = regex.exec(text)) !== null) {
			matches.push({ start: m.index, end: m.index + m[0].length });
		}
		if (matches.length === 0) return text;

		// Merge overlapping/adjacent ranges
		matches.sort((a, b) => a.start - b.start);
		const merged = [matches[0]];
		for (let i = 1; i < matches.length; i++) {
			const last = merged[merged.length - 1];
			if (matches[i].start <= last.end) {
				last.end = Math.max(last.end, matches[i].end);
			} else {
				merged.push(matches[i]);
			}
		}

		// Phase 2: build result by slicing original text and wrapping matched ranges
		const tag = '<mark class="bg-yellow-200 dark:bg-yellow-700/60 dark:text-yellow-100">';
		let result = '';
		let cursor = 0;
		for (const { start, end } of merged) {
			result += text.slice(cursor, start) + tag + text.slice(start, end) + '</mark>';
			cursor = end;
		}
		result += text.slice(cursor);
		return result;
	}

	function escapeRegex(str: string): string {
		return str.replace(/[.*+?^${}()|[\]\\]/g, '\\$&');
	}

	const highlighted = $derived(highlightText(text, highlightTerms));

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

<div class="text-xs sm:text-sm text-gray-700 dark:text-gray-300 leading-relaxed">
	{#if crumbs.length > 0}
		<div class="text-xs text-gray-400 dark:text-gray-500 mb-1">
			{#each crumbs as crumb, i}
				{#if i > 0}
					<span class="mx-1">&gt;</span>
				{/if}
				<a href={crumb.href} class="hover:text-blue-600 dark:hover:text-blue-400 hover:underline">{crumb.title}</a>
			{/each}
		</div>
	{/if}
	<div class="flex items-start justify-between gap-2 mb-1">
		<div class="flex-1 min-w-0 break-words">
			{@html highlighted}
		</div>
		<span class="text-xs text-gray-500 dark:text-gray-400 font-mono flex-shrink-0">
			{score.toFixed(3)}
		</span>
	</div>
</div>
