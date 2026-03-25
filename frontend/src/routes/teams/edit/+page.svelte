<script lang="ts">
	import { getTeamInfo, updateTeamInfo, type TeamMetadataEntry } from '$lib/api';

	let teamName = '';
	let code = '';
	let entries: { key: string; value: string }[] = [];
	let loaded = false;
	let loading = false;
	let saving = false;
	let message = '';
	let error = '';

	async function load() {
		if (!teamName.trim()) return;
		loading = true;
		error = '';
		message = '';
		try {
			const result = await getTeamInfo(teamName);
			entries = result.map((e) => ({ key: e.key, value: e.value }));
			loaded = true;
		} catch (e) {
			error = e instanceof Error ? e.message : 'Failed to load';
		} finally {
			loading = false;
		}
	}

	function addEntry() {
		entries = [...entries, { key: '', value: '' }];
	}

	function removeEntry(index: number) {
		entries = entries.filter((_, i) => i !== index);
	}

	async function save() {
		if (!code.trim()) {
			error = 'Please enter your team code';
			return;
		}
		saving = true;
		error = '';
		message = '';
		try {
			const result = await updateTeamInfo(teamName, code, entries);
			message = result;
		} catch (e) {
			error = e instanceof Error ? e.message : 'Failed to save';
		} finally {
			saving = false;
		}
	}
</script>

<svelte:head>
	<title>Edit Team Info</title>
</svelte:head>

<div class="container">
	<h1>Edit Team Info</h1>

	<div class="load-section">
		<label>
			Team Name
			<input type="text" bind:value={teamName} placeholder="e.g. TIGERs Mannheim" />
		</label>
		<label>
			Team Code
			<input type="password" bind:value={code} placeholder="Your team code" />
		</label>
		<button onclick={load} disabled={loading || !teamName.trim()}>
			{loading ? 'Loading...' : 'Load'}
		</button>
	</div>

	{#if loaded}
		<div class="entries-section">
			<h2>Metadata</h2>
			{#each entries as entry, i}
				<div class="entry-row">
					<input type="text" bind:value={entry.key} placeholder="key (e.g. github)" />
					<input type="text" bind:value={entry.value} placeholder="value (e.g. https://github.com/...)" />
					<button class="remove" onclick={() => removeEntry(i)}>Remove</button>
				</div>
			{/each}
			<button onclick={addEntry}>+ Add Entry</button>

			<div class="save-section">
				<button onclick={save} disabled={saving}>
					{saving ? 'Saving...' : 'Save Changes'}
				</button>
			</div>
		</div>
	{/if}

	{#if message}
		<div class="message success">{message}</div>
	{/if}
	{#if error}
		<div class="message error">{error}</div>
	{/if}
</div>

<style>
	.container {
		max-width: 700px;
		margin: 2rem auto;
		padding: 0 1rem;
	}
	.load-section, .entries-section, .save-section {
		margin: 1.5rem 0;
	}
	label {
		display: block;
		margin-bottom: 0.5rem;
	}
	input[type='text'], input[type='password'] {
		width: 100%;
		padding: 0.5rem;
		margin-top: 0.25rem;
		box-sizing: border-box;
	}
	.entry-row {
		display: flex;
		gap: 0.5rem;
		margin-bottom: 0.5rem;
		align-items: center;
	}
	.entry-row input:first-child {
		flex: 1;
	}
	.entry-row input:nth-child(2) {
		flex: 3;
	}
	button {
		padding: 0.5rem 1rem;
		cursor: pointer;
		margin-top: 0.5rem;
	}
	.remove {
		background: #c33;
		color: white;
		border: none;
		padding: 0.4rem 0.8rem;
	}
	.message {
		padding: 0.75rem;
		margin-top: 1rem;
		border-radius: 4px;
	}
	.success {
		background: #d4edda;
		color: #155724;
	}
	.error {
		background: #f8d7da;
		color: #721c24;
	}
</style>
