import type { PageLoad } from './$types';
import { getPaper } from '$lib/api';
import { error } from '@sveltejs/kit';

export const load: PageLoad = async ({ params }) => {
	try {
		// Parse the ID format: league__year__team__index
		const parts = params.id.split('__');
		if (parts.length !== 4) {
			throw error(400, 'Invalid paper ID format. Expected: league__year__team__index');
		}

		const [league, yearStr, team, indexStr] = parts;
		const year = parseInt(yearStr, 10);
		const index = parseInt(indexStr, 10);

		if (isNaN(year) || isNaN(index)) {
			throw error(400, 'Invalid year or index in paper ID');
		}

		// Fetch the paper content (markdown)
		const content = await getPaper(league, year, team, index);

		return {
			content,
			metadata: {
				league,
				team,
				year,
				index
			}
		};
	} catch (err) {
		console.error('Error loading paper:', err);
		throw error(404, 'Paper not found');
	}
};
