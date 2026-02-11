<script lang="ts">
	interface Props {
		text: string;
		query: string;
		score: number;
	}

	let { text, query, score }: Props = $props();

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
</script>

<div class="text-xs sm:text-sm text-gray-700 leading-relaxed">
	<div class="flex items-start justify-between gap-2 mb-1">
		<div class="flex-1 min-w-0 break-words">
			{@html highlighted}
		</div>
		<span class="text-xs text-gray-500 font-mono flex-shrink-0">
			{score.toFixed(3)}
		</span>
	</div>
</div>
