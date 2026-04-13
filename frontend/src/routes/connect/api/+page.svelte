<script lang="ts">
	import { page } from '$app/stores';
	import type { League } from '$lib/types';

	const BASE_URL = 'https://web.emielsteerneman.nl/api';

	let leagues: League[] = $derived($page.data.leagues || []);
	let years: number[] = $derived($page.data.years || []);

	const endpoints = [
		{ method: 'GET', path: '/api/search?query=<query>&league=&year=&team=&content_type=&search_type=', desc: 'Search across all papers using hybrid semantic+keyword search' },
		{ method: 'GET', path: '/api/papers?league=&year=&team=', desc: 'List papers, optionally filtered by league, year, or team' },
		{ method: 'GET', path: '/api/papers/{paper_lyt}/toc', desc: 'Get the table of contents for a paper' },
		{ method: 'GET', path: '/api/papers/{paper_lyt}/abstract', desc: 'Get the abstract of a paper' },
		{ method: 'GET', path: '/api/papers/{paper_lyt}/references', desc: 'Get the references/bibliography of a paper' },
		{ method: 'GET', path: '/api/papers/{paper_lyt}/info', desc: 'Get paper metadata: title, authors, institutions, URLs' },
		{ method: 'GET', path: '/api/papers/{paper_lyt}/paragraph/{seq}', desc: 'Get a specific paragraph by content sequence number' },
		{ method: 'GET', path: '/api/papers/{paper_lyt}/table/{seq}', desc: 'Get a specific table by content sequence number' },
		{ method: 'GET', path: '/api/papers/{paper_lyt}/image/{seq}', desc: 'Get a specific image by content sequence number' },
		{ method: 'GET', path: '/api/teams', desc: 'List all teams in the corpus' },
		{ method: 'GET', path: '/api/leagues', desc: 'List all leagues' },
		{ method: 'GET', path: '/api/years?league=&team=', desc: 'List all years, optionally filtered by league or team' },
		{ method: 'GET', path: '/api/registry/team/{name}', desc: 'Get team metadata: GitHub, website, social links' },
		{ method: 'GET', path: '/api/registry/league/{name}', desc: 'Get league metadata: official site, GitHub org, rules, community links' },
	];

	const examples = [
		{
			title: 'Search papers',
			desc: 'Search for papers about ball detection in the Small Size League',
			path: '/api/search?query=ball+detection&league=soccer_smallsize',
			python: `import requests

response = requests.get("${BASE_URL}/search", params={
    "query": "ball detection",
    "league": "soccer_smallsize"
})
results = response.json()["data"]

for chunk in results["chunks"]:
    print(f'{chunk["paper_lyt"]}: {chunk["title"]}')
    print(f'  {chunk["text"][:120]}...')`,
			curl: `curl -s "${BASE_URL}/search?query=ball+detection&league=soccer_smallsize" | python3 -m json.tool`,
		},
		{
			title: 'Read a paper abstract',
			desc: 'Get the abstract of a specific paper using its paper_lyt identifier',
			path: '/api/papers/soccer_smallsize__2024__RoboTeam_Twente/abstract',
			python: `import requests

paper = "soccer_smallsize__2024__RoboTeam_Twente"
response = requests.get(f"${BASE_URL}/papers/{paper}/abstract")
data = response.json()["data"]

print(data)`,
			curl: `curl -s "${BASE_URL}/papers/soccer_smallsize__2024__RoboTeam_Twente/abstract" | python3 -m json.tool`,
		},
		{
			title: 'List teams',
			desc: 'Get all teams in the corpus',
			path: '/api/teams',
			python: `import requests

response = requests.get("${BASE_URL}/teams")
teams = response.json()["data"]

for team in teams:
    print(f'{team["name"]:30} {team["name_pretty"]}')`,
			curl: `curl -s "${BASE_URL}/teams" | python3 -m json.tool`,
		},
	];
</script>

<svelte:head>
	<title>Connect your scripts — TDP Browser</title>
</svelte:head>

<div class="min-h-screen bg-gray-50 dark:bg-gray-950 py-8 px-4">
	<div class="max-w-2xl mx-auto">
		<!-- Tab navigation -->
		<div class="flex space-x-1 mb-6 bg-gray-200 dark:bg-gray-800 rounded-lg p-1 max-w-xs">
			<a href="/connect/mcp" class="flex-1 text-center px-3 py-1.5 text-sm rounded-md text-gray-600 dark:text-gray-400 hover:text-gray-900 dark:hover:text-gray-200 transition-colors">
				Your AI
			</a>
			<span class="flex-1 text-center px-3 py-1.5 text-sm rounded-md bg-white dark:bg-gray-700 text-gray-900 dark:text-gray-100 font-medium shadow-sm">
				Your scripts
			</span>
		</div>

		<!-- Header -->
		<div class="mb-8">
			<h1 class="text-2xl font-bold text-gray-900 dark:text-gray-100">Connect your scripts</h1>
			<p class="mt-2 text-sm text-gray-500 dark:text-gray-400 leading-relaxed">
				The TDP Search REST API gives you programmatic access to 2000+ RoboCup Team Description Papers.
				No authentication required. All endpoints return JSON.
			</p>
		</div>

		<!-- Base URL -->
		<div class="bg-white dark:bg-gray-900 rounded-lg border border-gray-200 dark:border-gray-800 p-6 shadow-sm mb-6">
			<h2 class="text-lg font-semibold text-gray-900 dark:text-gray-100 mb-2">Base URL</h2>
			<code class="block px-3 py-2 bg-gray-100 dark:bg-gray-800 rounded-lg text-sm font-mono text-gray-900 dark:text-gray-100 select-all">
				{BASE_URL}
			</code>
			<p class="mt-2 text-xs text-gray-500 dark:text-gray-400">
				All endpoints return JSON. No authentication required.
			</p>
		</div>

		<!-- Quick examples -->
		{#each examples as example}
			<div class="bg-white dark:bg-gray-900 rounded-lg border border-gray-200 dark:border-gray-800 p-6 shadow-sm mb-6">
				<h2 class="text-lg font-semibold text-gray-900 dark:text-gray-100 mb-1">{example.title}</h2>
				<p class="text-sm text-gray-500 dark:text-gray-400 mb-1">{example.desc}</p>
				<code class="text-xs text-gray-500 dark:text-gray-400 font-mono">{example.path}</code>

				<div class="mt-4 space-y-3">
					<div>
						<p class="text-xs font-medium text-gray-500 dark:text-gray-400 mb-1">Python</p>
						<pre class="px-3 py-2 bg-gray-100 dark:bg-gray-800 rounded-lg text-xs font-mono text-gray-900 dark:text-gray-100 overflow-x-auto">{example.python}</pre>
					</div>
					<div>
						<p class="text-xs font-medium text-gray-500 dark:text-gray-400 mb-1">curl</p>
						<pre class="px-3 py-2 bg-gray-100 dark:bg-gray-800 rounded-lg text-xs font-mono text-gray-900 dark:text-gray-100 overflow-x-auto">{example.curl}</pre>
					</div>
				</div>
			</div>
		{/each}

		<!-- Filter values -->
		<div class="bg-white dark:bg-gray-900 rounded-lg border border-gray-200 dark:border-gray-800 p-6 shadow-sm mb-6">
			<h2 class="text-lg font-semibold text-gray-900 dark:text-gray-100 mb-4">Filter values</h2>

			<p class="text-sm text-gray-600 dark:text-gray-400 mb-4">
				Use these exact values when filtering by league, year, or team. The <code class="text-xs bg-gray-100 dark:bg-gray-800 px-1 py-0.5 rounded">paper_lyt</code> identifier
				follows the format <code class="text-xs bg-gray-100 dark:bg-gray-800 px-1 py-0.5 rounded">{'{league}__{year}__{team}'}</code> (double underscore), e.g.
				<code class="text-xs bg-gray-100 dark:bg-gray-800 px-1 py-0.5 rounded">soccer_smallsize__2024__RoboTeam_Twente</code>.
			</p>

			<div class="mb-4">
				<h3 class="text-sm font-medium text-gray-900 dark:text-gray-100 mb-2">Leagues</h3>
				<div class="flex flex-wrap gap-1.5">
					{#each leagues as league}
						<code class="text-xs bg-gray-100 dark:bg-gray-800 px-2 py-1 rounded text-gray-700 dark:text-gray-300">{league.name}</code>
					{/each}
				</div>
			</div>

			<div>
				<h3 class="text-sm font-medium text-gray-900 dark:text-gray-100 mb-2">Years</h3>
				<div class="flex flex-wrap gap-1.5">
					{#each years as year}
						<code class="text-xs bg-gray-100 dark:bg-gray-800 px-2 py-1 rounded text-gray-700 dark:text-gray-300">{year}</code>
					{/each}
				</div>
			</div>
		</div>

		<!-- All endpoints -->
		<div class="bg-white dark:bg-gray-900 rounded-lg border border-gray-200 dark:border-gray-800 p-6 shadow-sm mb-6">
			<h2 class="text-lg font-semibold text-gray-900 dark:text-gray-100 mb-4">All endpoints</h2>
			<div class="space-y-2">
				{#each endpoints as endpoint}
					<div class="flex items-start gap-2 text-xs font-mono">
						<span class="text-green-600 dark:text-green-400 font-semibold flex-shrink-0 w-8">{endpoint.method}</span>
						<span class="text-gray-900 dark:text-gray-100">{endpoint.path}</span>
					</div>
					<p class="text-xs text-gray-500 dark:text-gray-400 ml-10 -mt-1 mb-2">{endpoint.desc}</p>
				{/each}
			</div>
			<p class="mt-4 text-xs text-gray-500 dark:text-gray-400">
				Machine-readable endpoint list available at <a href="/api" target="_blank" class="text-blue-500 hover:underline">/api</a>.
			</p>
		</div>

		<!-- paper_lyt explainer -->
		<div class="bg-blue-50 dark:bg-blue-900/20 rounded-lg border border-blue-200 dark:border-blue-800 p-6">
			<h2 class="text-lg font-semibold text-gray-900 dark:text-gray-100 mb-2">Paper identifiers</h2>
			<p class="text-sm text-gray-600 dark:text-gray-400 mb-3">
				Every paper has a unique <code class="text-xs bg-blue-100 dark:bg-blue-800/40 px-1 py-0.5 rounded">paper_lyt</code> identifier built from its league, year, and team name, separated by double underscores:
			</p>
			<code class="block px-3 py-2 bg-blue-100 dark:bg-blue-800/40 rounded-lg text-sm font-mono text-gray-900 dark:text-gray-100">
				{'{league}__{year}__{team}'}
			</code>
			<p class="mt-2 text-xs text-gray-500 dark:text-gray-400">
				Example: <code class="bg-blue-100 dark:bg-blue-800/40 px-1 py-0.5 rounded">soccer_smallsize__2024__RoboTeam_Twente</code>
			</p>
		</div>
	</div>
</div>
