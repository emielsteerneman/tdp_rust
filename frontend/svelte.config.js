import adapter from '@sveltejs/adapter-static';
import { vitePreprocess } from '@sveltejs/vite-plugin-svelte';

/** @type {import('@sveltejs/kit').Config} */
const config = {
	preprocess: vitePreprocess(),
	kit: {
		adapter: adapter({
			fallback: 'index.html' // SPA fallback for non-prerendered (dynamic) routes
		}),
		prerender: {
			// A single missing/broken paper must not fail the whole corpus build.
			handleHttpError: 'warn',
			handleMissingId: 'warn'
		}
	}
};

export default config;
