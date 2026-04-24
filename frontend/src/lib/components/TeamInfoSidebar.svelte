<script lang="ts">
	import type { RegistryEntry } from '$lib/api';
	import { getLeagueBadgeColor } from '$lib/leagueColors';

	interface Props {
		teamName: string;
		leagueMachine: string;
		year: number;
		entries: RegistryEntry[];
	}

	let { teamName, leagueMachine, year, entries }: Props = $props();

	const leagueMajor = $derived(leagueMachine.split('_')[0] ?? '');
	const leagueBadge = $derived(getLeagueBadgeColor(leagueMajor));
	const leagueLabel = $derived(leagueMachine.replace(/_/g, ' '));

	const primaryKeys = ['website', 'github'];
	const socialKeys = ['instagram', 'facebook', 'twitter', 'youtube'];
	const infoKeys = ['university', 'country'];

	const grouped = $derived.by(() => {
		const primary: RegistryEntry[] = [];
		const social: RegistryEntry[] = [];
		const info: RegistryEntry[] = [];
		const other: RegistryEntry[] = [];

		for (const entry of entries) {
			if (primaryKeys.includes(entry.key)) primary.push(entry);
			else if (socialKeys.includes(entry.key)) social.push(entry);
			else if (infoKeys.includes(entry.key)) info.push(entry);
			else other.push(entry);
		}

		return { primary, social, info, other };
	});

	const hasRegistryLinks = $derived(entries.length > 0);

	function isUrl(value: string): boolean {
		return value.startsWith('http://') || value.startsWith('https://');
	}

	function looksLikeUrl(value: string): boolean {
		return isUrl(value) || /^[a-zA-Z0-9][\w.-]+\.[a-z]{2,}(\/|$)/.test(value);
	}

	function ensureProtocol(url: string): string {
		if (url.startsWith('http://') || url.startsWith('https://')) return url;
		return `https://${url}`;
	}

	const labels: Record<string, string> = {
		github: 'GitHub',
		website: 'Website',
		instagram: 'Instagram',
		facebook: 'Facebook',
		twitter: 'Twitter / X',
		youtube: 'YouTube',
		qualification_video: 'Qualification',
		university: 'University',
		country: 'Country'
	};

	function labelFor(key: string): string {
		return labels[key] ?? key.charAt(0).toUpperCase() + key.slice(1).replace(/_/g, ' ');
	}
</script>

{#snippet icon(key: string)}
	{#if key === 'github'}
		<svg viewBox="0 0 16 16" fill="currentColor"><path d="M8 0C3.58 0 0 3.58 0 8c0 3.54 2.29 6.53 5.47 7.59.4.07.55-.17.55-.38 0-.19-.01-.82-.01-1.49-2.01.37-2.53-.49-2.69-.94-.09-.23-.48-.94-.82-1.13-.28-.15-.68-.52-.01-.53.63-.01 1.08.58 1.23.82.72 1.21 1.87.87 2.33.66.07-.52.28-.87.51-1.07-1.78-.2-3.64-.89-3.64-3.95 0-.87.31-1.59.82-2.15-.08-.2-.36-1.02.08-2.12 0 0 .67-.21 2.2.82.64-.18 1.32-.27 2-.27.68 0 1.36.09 2 .27 1.53-1.04 2.2-.82 2.2-.82.44 1.1.16 1.92.08 2.12.51.56.82 1.27.82 2.15 0 3.07-1.87 3.75-3.65 3.95.29.25.54.73.54 1.48 0 1.07-.01 1.93-.01 2.2 0 .21.15.46.55.38A8.013 8.013 0 0016 8c0-4.42-3.58-8-8-8z"/></svg>
	{:else if key === 'website'}
		<svg viewBox="0 0 20 20" fill="currentColor"><path d="M10 18a8 8 0 100-16 8 8 0 000 16zM4.332 8.027a6.012 6.012 0 011.912-2.706C6.512 5.73 6.974 6 7.5 6A1.5 1.5 0 019 7.5V8a2 2 0 004 0 2 2 0 011.523-1.943A5.977 5.977 0 0116 10c0 .34-.028.675-.083 1H15a2 2 0 00-2 2v2.197A5.973 5.973 0 0110 16v-2a2 2 0 00-2-2 2 2 0 01-2-2 2 2 0 00-1.668-1.973z"/></svg>
	{:else if key === 'university'}
		<svg viewBox="0 0 20 20" fill="currentColor"><path d="M10 1l-9 4.5 3 1.5v4l6 3 6-3v-4l1.5-.75V13h1.5V6.25L10 1zm0 2.18L15.36 6 10 8.82 4.64 6 10 3.18z"/></svg>
	{:else if key === 'country'}
		<svg viewBox="0 0 20 20" fill="currentColor"><path fill-rule="evenodd" d="M9.69 18.933l.003.001C9.89 19.02 10 19 10 19s.11.02.308-.066l.002-.001.006-.003.018-.008a5.741 5.741 0 00.281-.14c.186-.096.446-.24.757-.433.62-.384 1.445-.966 2.274-1.765C15.302 14.988 17 12.493 17 9A7 7 0 103 9c0 3.492 1.698 5.988 3.355 7.584a13.731 13.731 0 002.273 1.765 11.842 11.842 0 00.976.544l.062.029.018.008.006.003zM10 11.25a2.25 2.25 0 100-4.5 2.25 2.25 0 000 4.5z" clip-rule="evenodd"/></svg>
	{:else}
		<svg viewBox="0 0 20 20" fill="currentColor"><path d="M12.232 4.232a2.5 2.5 0 013.536 3.536l-1.225 1.224a.75.75 0 001.061 1.06l1.224-1.224a4 4 0 00-5.656-5.656l-3 3a4 4 0 00.225 5.865.75.75 0 00.977-1.138 2.5 2.5 0 01-.142-3.667l3-3z"/><path d="M11.603 7.963a.75.75 0 00-.977 1.138 2.5 2.5 0 01.142 3.667l-3 3a2.5 2.5 0 01-3.536-3.536l1.225-1.224a.75.75 0 00-1.061-1.06l-1.224 1.224a4 4 0 005.656 5.656l3-3a4 4 0 00-.225-5.865z"/></svg>
	{/if}
{/snippet}

{#snippet linkItem(href: string, key: string, label: string)}
	<a
		{href}
		target="_blank"
		rel="noopener noreferrer"
		class="flex items-center gap-2 text-sm text-gray-600 dark:text-gray-300 hover:text-blue-600 dark:hover:text-blue-400 transition-colors group"
	>
		<span class="flex-shrink-0 w-4 h-4 text-gray-400 dark:text-gray-500 group-hover:text-blue-500">
			{@render icon(key)}
		</span>
		<span class="truncate">{label}</span>
	</a>
{/snippet}

{#snippet textItem(key: string, value: string)}
	<div class="flex items-center gap-2 text-sm text-gray-500 dark:text-gray-400">
		<span class="flex-shrink-0 w-4 h-4">
			{@render icon(key)}
		</span>
		<span class="truncate">{value}</span>
	</div>
{/snippet}

<aside
	class="hidden lg:block w-52 flex-shrink-0 bg-white dark:bg-gray-900 border-l border-gray-200 dark:border-gray-800 h-screen sticky top-16 overflow-y-auto"
>
	<div class="p-4">
		<h2
			class="text-sm font-semibold text-gray-500 dark:text-gray-400 uppercase tracking-wider mb-2"
		>
			Team
		</h2>
		<p class="text-base font-medium text-gray-900 dark:text-gray-100 break-words mb-3">
			{teamName}
		</p>

		<div class="flex flex-wrap items-center gap-2 mb-4">
			<span class="inline-flex px-2 py-0.5 rounded-full text-xs font-medium {leagueBadge}">
				{leagueLabel}
			</span>
			<span class="text-xs text-gray-500 dark:text-gray-400 font-medium">{year}</span>
		</div>

		{#if hasRegistryLinks}
			<hr class="border-gray-200 dark:border-gray-700 mb-3" />
			<h3 class="text-xs font-semibold text-gray-500 dark:text-gray-400 uppercase tracking-wider mb-2">Links</h3>

			<div class="space-y-1.5">
				{#each grouped.primary as entry}
					{@render linkItem(ensureProtocol(entry.value), entry.key, labelFor(entry.key))}
				{/each}
				{#each grouped.social as entry}
					{@render linkItem(ensureProtocol(entry.value), entry.key, labelFor(entry.key))}
				{/each}
				{#each grouped.other as entry}
					{#if looksLikeUrl(entry.value)}
						{@render linkItem(ensureProtocol(entry.value), entry.key, labelFor(entry.key))}
					{:else}
						{@render textItem(entry.key, entry.value)}
					{/if}
				{/each}
				{#each grouped.info as entry}
					{@render textItem(entry.key, entry.value)}
				{/each}
			</div>
		{:else}
			<hr class="border-gray-200 dark:border-gray-700 mb-3" />
			<p class="text-xs text-gray-400 dark:text-gray-500">
				No team links yet.
				<a href="/teams/edit" class="text-blue-500 hover:text-blue-600 dark:hover:text-blue-400">Add info</a>
			</p>
		{/if}
	</div>
</aside>
