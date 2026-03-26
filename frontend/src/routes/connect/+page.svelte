<svelte:head>
	<title>Connect your AI — TDP Browser</title>
</svelte:head>

<div class="min-h-screen bg-gray-50 dark:bg-gray-950 py-8 px-4">
	<div class="max-w-2xl mx-auto">
		<div class="mb-8">
			<h1 class="text-2xl font-bold text-gray-900 dark:text-gray-100">Connect your AI</h1>
			<p class="mt-2 text-sm text-gray-500 dark:text-gray-400 leading-relaxed">
				Add the TDP Search MCP server to your AI assistant and get access to 2000+ RoboCup Team Description Papers
				right inside your conversations. Search, read papers, explore teams — all through natural language.
			</p>
		</div>

		<!-- Server URL -->
		<div class="bg-white dark:bg-gray-900 rounded-lg border border-gray-200 dark:border-gray-800 p-6 shadow-sm mb-6">
			<h2 class="text-lg font-semibold text-gray-900 dark:text-gray-100 mb-2">MCP Server URL</h2>
			<code class="block px-3 py-2 bg-gray-100 dark:bg-gray-800 rounded-lg text-sm font-mono text-gray-900 dark:text-gray-100 select-all">
				https://mcp.emielsteerneman.nl/mcp
			</code>
			<p class="mt-2 text-xs text-gray-500 dark:text-gray-400">
				Transport: Streamable HTTP — no authentication required.
			</p>
		</div>

		<!-- Setup guides -->
		<div class="bg-white dark:bg-gray-900 rounded-lg border border-gray-200 dark:border-gray-800 p-6 shadow-sm mb-6">
			<h2 class="text-lg font-semibold text-gray-900 dark:text-gray-100 mb-4">Setup guides</h2>
			<p class="text-sm text-gray-600 dark:text-gray-400 mb-4">
				Each AI client has its own way of adding MCP servers. Follow your client's guide and use the URL above.
			</p>
			<div class="space-y-3">
				<a href="https://modelcontextprotocol.io/quickstart/user" target="_blank" rel="noopener noreferrer"
					class="flex items-center justify-between px-4 py-3 rounded-lg border border-gray-200 dark:border-gray-700 hover:bg-gray-50 dark:hover:bg-gray-800 transition-colors group">
					<div>
						<p class="text-sm font-medium text-gray-900 dark:text-gray-100">Claude Desktop</p>
						<p class="text-xs text-gray-500 dark:text-gray-400">modelcontextprotocol.io</p>
					</div>
					<svg class="w-4 h-4 text-gray-400 group-hover:text-gray-600 dark:group-hover:text-gray-300 transition-colors" fill="none" stroke="currentColor" viewBox="0 0 24 24">
						<path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M10 6H6a2 2 0 00-2 2v10a2 2 0 002 2h10a2 2 0 002-2v-4M14 4h6m0 0v6m0-6L10 14" />
					</svg>
				</a>
				<a href="https://docs.anthropic.com/en/docs/claude-code/mcp" target="_blank" rel="noopener noreferrer"
					class="flex items-center justify-between px-4 py-3 rounded-lg border border-gray-200 dark:border-gray-700 hover:bg-gray-50 dark:hover:bg-gray-800 transition-colors group">
					<div>
						<p class="text-sm font-medium text-gray-900 dark:text-gray-100">Claude Code</p>
						<p class="text-xs text-gray-500 dark:text-gray-400">docs.anthropic.com</p>
					</div>
					<svg class="w-4 h-4 text-gray-400 group-hover:text-gray-600 dark:group-hover:text-gray-300 transition-colors" fill="none" stroke="currentColor" viewBox="0 0 24 24">
						<path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M10 6H6a2 2 0 00-2 2v10a2 2 0 002 2h10a2 2 0 002-2v-4M14 4h6m0 0v6m0-6L10 14" />
					</svg>
				</a>
				<a href="https://docs.cursor.com/context/model-context-protocol" target="_blank" rel="noopener noreferrer"
					class="flex items-center justify-between px-4 py-3 rounded-lg border border-gray-200 dark:border-gray-700 hover:bg-gray-50 dark:hover:bg-gray-800 transition-colors group">
					<div>
						<p class="text-sm font-medium text-gray-900 dark:text-gray-100">Cursor</p>
						<p class="text-xs text-gray-500 dark:text-gray-400">docs.cursor.com</p>
					</div>
					<svg class="w-4 h-4 text-gray-400 group-hover:text-gray-600 dark:group-hover:text-gray-300 transition-colors" fill="none" stroke="currentColor" viewBox="0 0 24 24">
						<path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M10 6H6a2 2 0 00-2 2v10a2 2 0 002 2h10a2 2 0 002-2v-4M14 4h6m0 0v6m0-6L10 14" />
					</svg>
				</a>
				<a href="https://docs.windsurf.com/windsurf/cascade/mcp" target="_blank" rel="noopener noreferrer"
					class="flex items-center justify-between px-4 py-3 rounded-lg border border-gray-200 dark:border-gray-700 hover:bg-gray-50 dark:hover:bg-gray-800 transition-colors group">
					<div>
						<p class="text-sm font-medium text-gray-900 dark:text-gray-100">Windsurf</p>
						<p class="text-xs text-gray-500 dark:text-gray-400">docs.windsurf.com</p>
					</div>
					<svg class="w-4 h-4 text-gray-400 group-hover:text-gray-600 dark:group-hover:text-gray-300 transition-colors" fill="none" stroke="currentColor" viewBox="0 0 24 24">
						<path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M10 6H6a2 2 0 00-2 2v10a2 2 0 002 2h10a2 2 0 002-2v-4M14 4h6m0 0v6m0-6L10 14" />
					</svg>
				</a>
				<a href="https://geminicli.com/docs/tools/mcp-server/" target="_blank" rel="noopener noreferrer"
					class="flex items-center justify-between px-4 py-3 rounded-lg border border-gray-200 dark:border-gray-700 hover:bg-gray-50 dark:hover:bg-gray-800 transition-colors group">
					<div>
						<p class="text-sm font-medium text-gray-900 dark:text-gray-100">Gemini CLI</p>
						<p class="text-xs text-gray-500 dark:text-gray-400">geminicli.com</p>
					</div>
					<svg class="w-4 h-4 text-gray-400 group-hover:text-gray-600 dark:group-hover:text-gray-300 transition-colors" fill="none" stroke="currentColor" viewBox="0 0 24 24">
						<path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M10 6H6a2 2 0 00-2 2v10a2 2 0 002 2h10a2 2 0 002-2v-4M14 4h6m0 0v6m0-6L10 14" />
					</svg>
				</a>
				<a href="https://developers.openai.com/codex/mcp" target="_blank" rel="noopener noreferrer"
					class="flex items-center justify-between px-4 py-3 rounded-lg border border-gray-200 dark:border-gray-700 hover:bg-gray-50 dark:hover:bg-gray-800 transition-colors group">
					<div>
						<p class="text-sm font-medium text-gray-900 dark:text-gray-100">OpenAI Codex</p>
						<p class="text-xs text-gray-500 dark:text-gray-400">developers.openai.com</p>
					</div>
					<svg class="w-4 h-4 text-gray-400 group-hover:text-gray-600 dark:group-hover:text-gray-300 transition-colors" fill="none" stroke="currentColor" viewBox="0 0 24 24">
						<path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M10 6H6a2 2 0 00-2 2v10a2 2 0 002 2h10a2 2 0 002-2v-4M14 4h6m0 0v6m0-6L10 14" />
					</svg>
				</a>
			<a href="https://mistral.ai/news/le-chat-mcp-connectors-memories" target="_blank" rel="noopener noreferrer"
					class="flex items-center justify-between px-4 py-3 rounded-lg border border-gray-200 dark:border-gray-700 hover:bg-gray-50 dark:hover:bg-gray-800 transition-colors group">
					<div>
						<p class="text-sm font-medium text-gray-900 dark:text-gray-100">Mistral Le Chat</p>
						<p class="text-xs text-gray-500 dark:text-gray-400">mistral.ai</p>
					</div>
					<svg class="w-4 h-4 text-gray-400 group-hover:text-gray-600 dark:group-hover:text-gray-300 transition-colors" fill="none" stroke="currentColor" viewBox="0 0 24 24">
						<path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M10 6H6a2 2 0 00-2 2v10a2 2 0 002 2h10a2 2 0 002-2v-4M14 4h6m0 0v6m0-6L10 14" />
					</svg>
				</a>
			</div>
		</div>

		<!-- What you get -->
		<div class="bg-white dark:bg-gray-900 rounded-lg border border-gray-200 dark:border-gray-800 p-6 shadow-sm mb-6">
			<h2 class="text-lg font-semibold text-gray-900 dark:text-gray-100 mb-4">What you get</h2>
			<div class="space-y-3">
				<div class="flex items-start gap-3">
					<svg class="w-5 h-5 text-green-500 mt-0.5 flex-shrink-0" fill="none" stroke="currentColor" viewBox="0 0 24 24">
						<path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M5 13l4 4L19 7" />
					</svg>
					<div>
						<p class="text-sm font-medium text-gray-900 dark:text-gray-100">Search</p>
						<p class="text-xs text-gray-500 dark:text-gray-400">Hybrid semantic + keyword search across 2000+ papers</p>
					</div>
				</div>
				<div class="flex items-start gap-3">
					<svg class="w-5 h-5 text-green-500 mt-0.5 flex-shrink-0" fill="none" stroke="currentColor" viewBox="0 0 24 24">
						<path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M5 13l4 4L19 7" />
					</svg>
					<div>
						<p class="text-sm font-medium text-gray-900 dark:text-gray-100">Read full papers</p>
						<p class="text-xs text-gray-500 dark:text-gray-400">Access complete paper contents, sections, and abstracts</p>
					</div>
				</div>
				<div class="flex items-start gap-3">
					<svg class="w-5 h-5 text-green-500 mt-0.5 flex-shrink-0" fill="none" stroke="currentColor" viewBox="0 0 24 24">
						<path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M5 13l4 4L19 7" />
					</svg>
					<div>
						<p class="text-sm font-medium text-gray-900 dark:text-gray-100">Browse teams, leagues & years</p>
						<p class="text-xs text-gray-500 dark:text-gray-400">List and filter the entire RoboCup TDP corpus</p>
					</div>
				</div>
				<div class="flex items-start gap-3">
					<svg class="w-5 h-5 text-green-500 mt-0.5 flex-shrink-0" fill="none" stroke="currentColor" viewBox="0 0 24 24">
						<path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M5 13l4 4L19 7" />
					</svg>
					<div>
						<p class="text-sm font-medium text-gray-900 dark:text-gray-100">Natural language</p>
						<p class="text-xs text-gray-500 dark:text-gray-400">Just ask your AI — "What approaches do teams use for ball detection?"</p>
					</div>
				</div>
			</div>
		</div>

		<!-- Try it -->
		<div class="bg-blue-50 dark:bg-blue-900/20 rounded-lg border border-blue-200 dark:border-blue-800 p-6">
			<h2 class="text-lg font-semibold text-gray-900 dark:text-gray-100 mb-2">Try it out</h2>
			<p class="text-sm text-gray-600 dark:text-gray-400 mb-3">
				Once connected, try asking your AI something like:
			</p>
			<ul class="space-y-2 text-sm text-gray-700 dark:text-gray-300">
				<li class="flex items-start gap-2">
					<span class="text-blue-500 mt-0.5">→</span>
					<span>"What vision systems do RoboCup Small Size League teams use?"</span>
				</li>
				<li class="flex items-start gap-2">
					<span class="text-blue-500 mt-0.5">→</span>
					<span>"Compare the motion planning approaches of TIGERs Mannheim and RoboTeam Twente"</span>
				</li>
				<li class="flex items-start gap-2">
					<span class="text-blue-500 mt-0.5">→</span>
					<span>"Show me the latest papers from the Humanoid league"</span>
				</li>
			</ul>
		</div>
	</div>
</div>
