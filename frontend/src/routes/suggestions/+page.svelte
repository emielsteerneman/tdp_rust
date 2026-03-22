<script lang="ts">
	import { submitSuggestion } from '$lib/api';

	let message = $state('');
	let status: 'idle' | 'submitting' | 'success' | 'error' = $state('idle');
	let errorMessage = $state('');

	const maxLength = 2000;
	let charCount = $derived(message.length);
	let canSubmit = $derived(message.trim().length > 0 && message.length <= maxLength && (status as string) !== 'submitting');

	async function handleSubmit() {
		if (!canSubmit) return;

		status = 'submitting';
		errorMessage = '';

		try {
			await submitSuggestion(message);
			status = 'success';
			message = '';
		} catch (e) {
			status = 'error';
			errorMessage = e instanceof Error ? e.message : 'Something went wrong';
		}
	}
</script>

<div class="max-w-2xl mx-auto px-4 py-8 sm:py-12">
	<h1 class="text-2xl font-semibold text-gray-900 dark:text-gray-100 mb-2">Suggestions</h1>
	<p class="text-gray-600 dark:text-gray-400 mb-6">
		Have an idea to improve TDP Browser? Found a bug? Let us know!
	</p>

	{#if status === 'success'}
		<div class="bg-green-50 dark:bg-green-900/30 border border-green-200 dark:border-green-800 rounded-lg p-4 mb-6">
			<p class="text-green-800 dark:text-green-300">Thank you for your suggestion!</p>
		</div>
		<button
			onclick={() => { status = 'idle'; }}
			class="text-blue-600 dark:text-blue-400 hover:underline"
		>
			Submit another
		</button>
	{:else}
		<form onsubmit={e => { e.preventDefault(); handleSubmit(); }}>
			<textarea
				bind:value={message}
				placeholder="Your suggestion or feedback..."
				rows="5"
				maxlength={maxLength}
				class="w-full px-4 py-3 border border-gray-300 dark:border-gray-700 rounded-lg bg-white dark:bg-gray-800 text-gray-900 dark:text-gray-100 placeholder-gray-400 dark:placeholder-gray-500 focus:ring-2 focus:ring-blue-500 focus:border-transparent resize-y"
			></textarea>

			<div class="flex items-center justify-between mt-2">
				<span class="text-sm text-gray-500 dark:text-gray-400">
					{charCount}/{maxLength}
				</span>

				<button
					type="submit"
					disabled={!canSubmit}
					class="px-4 py-2 bg-blue-600 text-white rounded-lg hover:bg-blue-700 disabled:opacity-50 disabled:cursor-not-allowed transition-colors"
				>
					{#if status === 'submitting'}
						Sending...
					{:else}
						Send Suggestion
					{/if}
				</button>
			</div>
		</form>

		{#if status === 'error'}
			<div class="bg-red-50 dark:bg-red-900/30 border border-red-200 dark:border-red-800 rounded-lg p-4 mt-4">
				<p class="text-red-800 dark:text-red-300">{errorMessage}</p>
			</div>
		{/if}
	{/if}
</div>
