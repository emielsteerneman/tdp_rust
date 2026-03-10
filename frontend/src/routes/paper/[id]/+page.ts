import type { PageLoad } from './$types';
import { error } from '@sveltejs/kit';

export const load: PageLoad = async ({ params, fetch }) => {
  const lyti = params.id;

  const response = await fetch(`/tdps/${lyti}.md`);
  if (!response.ok) {
    throw error(response.status, 'Paper not found');
  }

  const rawMarkdown = await response.text();

  return {
    rawMarkdown,
    lyti
  };
};
