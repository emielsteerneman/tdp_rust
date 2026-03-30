<script lang="ts">
	import { marked } from 'marked';
	import type { PageData } from './$types';
	import { preprocessMarkdown, extractHeadings, slugifyHeading } from '$lib/markdown';
	import TableOfContents from '$lib/components/TableOfContents.svelte';
	import TeamInfoSidebar from '$lib/components/TeamInfoSidebar.svelte';

	let { data }: { data: PageData } = $props();

	// Configure marked renderer to generate heading IDs matching extractHeadings output
	const renderer = new marked.Renderer();
	// @ts-ignore — marked v12 uses (text, level, raw) not the object form
	renderer.heading = (text: string, level: number) => {
		const id = slugifyHeading(text);
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

<div class="min-h-screen bg-gray-50 dark:bg-gray-950 flex">
	<TableOfContents {headings} {activeId} />

	<div class="flex-1 min-w-0">
		<div class="max-w-4xl mx-auto px-4 py-6 sm:py-8">
			<!-- Paper Content -->
			<div class="bg-white dark:bg-gray-900 rounded-lg shadow-sm border border-gray-200 dark:border-gray-800 p-4 sm:p-6 md:p-8 mb-4 sm:mb-6">
				<div class="flex justify-end mb-4">
					<a
						href="/pdfs/{data.lyti}.pdf"
						target="_blank"
						rel="noopener noreferrer"
						class="inline-flex items-center gap-2 px-4 py-2 bg-blue-600 hover:bg-blue-700 text-white text-sm font-medium rounded-lg transition-colors"
						onclick={() => {
							fetch(`/api/papers/${encodeURIComponent(data.lyti)}/pdf-open`, { method: 'POST' }).catch(() => {});
						}}
					>
						View Original PDF
					</a>
				</div>
				<article
					class="prose prose-gray dark:prose-invert prose-sm sm:prose-base max-w-none
						prose-headings:font-bold prose-headings:scroll-mt-24
						prose-h2:text-xl sm:prose-h2:text-2xl
						prose-h3:text-lg sm:prose-h3:text-xl
						prose-p:leading-relaxed
						prose-a:text-blue-600 hover:prose-a:text-blue-800 dark:prose-a:text-blue-400 dark:hover:prose-a:text-blue-300
						prose-img:rounded-lg prose-img:shadow-md prose-img:mx-auto"
				>
					{@html htmlContent}
				</article>
			</div>

		</div>
	</div>

	<TeamInfoSidebar
		teamName={data.teamPrettyName}
		leagueMachine={data.leagueMachine}
		year={data.year}
		paperUrls={data.paperInfo?.urls ?? []}
		entries={data.teamEntries}
	/>
</div>
