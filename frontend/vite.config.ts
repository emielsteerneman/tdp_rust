import { sveltekit } from '@sveltejs/kit/vite';
import tailwindcss from '@tailwindcss/vite';
import { defineConfig } from 'vite';

export default defineConfig({
	plugins: [tailwindcss(), sveltekit()],
	server: {
		allowedHosts: ['tdp.emielsteerneman.nl'],
		proxy: {
			'/api': {
				target: 'http://localhost:8081',
				changeOrigin: true
			}
		}
	}
});
