import adapter from '@sveltejs/adapter-static';
import { vitePreprocess } from '@sveltejs/vite-plugin-svelte';

/** @type {import('@sveltejs/kit').Config} */
const config = {
	preprocess: vitePreprocess(),
	kit: {
		adapter: adapter({
			fallback: '200.html' // SPA fallback for non-prerendered (dynamic) routes; distinct from the prerendered homepage's index.html
		}),
		prerender: {
			// A single missing/broken paper must not fail the whole corpus build.
			handleHttpError: 'warn',
			handleMissingId: 'warn'
		}
	}
};

export default config;
