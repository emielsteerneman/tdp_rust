<script lang="ts">
	import { marked } from 'marked';
	import type { PageData } from './$types';
	import { preprocessMarkdown, extractHeadings } from '$lib/markdown';
	import TableOfContents from '$lib/components/TableOfContents.svelte';

	let { data }: { data: PageData } = $props();

	// Configure marked renderer to generate heading IDs matching extractHeadings output
	const renderer = new marked.Renderer();
	// @ts-ignore — marked v12 uses (text, level, raw) not the object form
	renderer.heading = (text: string, level: number) => {
		const id = text
			.toLowerCase()
			.replace(/[^a-z0-9\s-]/g, '')
			.trim()
			.replace(/\s+/g, '-');
		return `<h${level} id="${id}">${text}</h${level}>`;
	};
	marked.use({ renderer });
	marked.setOptions({ gfm: true, breaks: false });

	// Pre-process markdown and extract headings
	const processed = $derived(preprocessMarkdown(data.rawMarkdown, data.lyti));
	const headings = $derived(extractHeadings(processed));
	const htmlContent = $derived(marked.parse(processed) as string);

	// Active heading tracking
	let activeId = $state('');

	function handleScroll() {
		const ids = headings.map((h) => h.id);
		for (let i = ids.length - 1; i >= 0; i--) {
			const el = document.getElementById(ids[i]);
			if (el && el.getBoundingClientRect().top <= 100) {
				activeId = ids[i];
				return;
			}
		}
		activeId = '';
	}
</script>

<svelte:window onscroll={handleScroll} />

<div class="min-h-screen bg-gray-50 flex">
	<TableOfContents {headings} {activeId} />

	<div class="flex-1 min-w-0">
		<div class="max-w-4xl mx-auto px-4 py-6 sm:py-8">
			<!-- Paper Content -->
			<div class="bg-white rounded-lg shadow-sm border border-gray-200 p-4 sm:p-6 md:p-8 mb-4 sm:mb-6">
				<article
					class="prose prose-gray prose-sm sm:prose-base max-w-none
						prose-headings:font-bold
						prose-h2:text-xl sm:prose-h2:text-2xl
						prose-h3:text-lg sm:prose-h3:text-xl
						prose-p:leading-relaxed
						prose-a:text-blue-600 hover:prose-a:text-blue-800
						prose-img:rounded-lg prose-img:shadow-md prose-img:mx-auto"
				>
					{@html htmlContent}
				</article>
			</div>

			<!-- Actions -->
			<div class="flex justify-center">
				<button
					disabled
					class="relative px-4 sm:px-6 py-2 sm:py-3 bg-gray-300 text-gray-500 rounded-lg font-medium cursor-not-allowed group text-sm sm:text-base"
					title="Coming soon"
				>
					View Original PDF
					<span
						class="absolute bottom-full left-1/2 transform -translate-x-1/2 mb-2 px-3 py-1 bg-gray-900 text-white text-xs rounded opacity-0 group-hover:opacity-100 transition-opacity whitespace-nowrap pointer-events-none"
					>
						Coming soon
					</span>
				</button>
			</div>
		</div>
	</div>
</div>
