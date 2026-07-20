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
			// Don't follow links found in rendered pages: paper pages come from the
			// [id] route's entries(), and the homepage/connect pages are explicit
			// prerender routes — so crawling adds nothing. It also avoids the crawler
			// crashing on malformed URLs in TDP source (e.g. bad autolinks like
			// "http://www.rctools.sourceforge.jp.2015") that appear across the corpus.
			crawl: false,
			// A single missing/broken paper must not fail the whole corpus build,
			// but any other error (esp. 5xx render errors) should fail the build loudly.
			handleHttpError: ({ status, path, referrer, message }) => {
				if (status === 404) {
					console.warn(`prerender: skipping ${path} (404, linked from ${referrer})`);
					return;
				}
				throw new Error(message);
			},
			handleMissingId: 'warn'
		}
	}
};

export default config;
