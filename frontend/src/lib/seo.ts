const PRETTY_LEAGUE: Record<string, string> = {
	soccer_smallsize: 'Soccer SmallSize',
	soccer_midsize: 'Soccer MiddleSize'
	// Non-exhaustive: unknown machine names fall back to a title-cased form below.
};

export function prettyLeague(machine: string): string {
	return (
		PRETTY_LEAGUE[machine] ??
		machine
			.split('_')
			.map((w) => w.charAt(0).toUpperCase() + w.slice(1))
			.join(' ')
	);
}

export function paperTitle(team: string, leagueMachine: string, year: number): string {
	return `${team} — ${prettyLeague(leagueMachine)} ${year} | TDPSearch`;
}

/** First ~155 chars of the markdown body, stripped of markup, for meta description. */
export function metaDescription(markdown: string, team: string, year: number): string {
	const text = markdown
		.replace(/^#.*$/gm, '') // drop heading lines
		.replace(/[#*_`>\[\]()!]/g, '') // drop common md punctuation
		.replace(/\s+/g, ' ')
		.trim();
	const fallback = `Team Description Paper by ${team} (${year}), on tdpsearch.com.`;
	const base = text.length > 0 ? text : fallback;
	return base.length > 155 ? base.slice(0, 152).trimEnd() + '…' : base;
}

export function articleJsonLd(opts: {
	title: string;
	description: string;
	url: string;
	team: string;
	year: number;
	league: string;
}): string {
	return JSON.stringify({
		'@context': 'https://schema.org',
		'@type': 'ScholarlyArticle',
		headline: opts.title,
		description: opts.description,
		url: opts.url,
		datePublished: String(opts.year),
		author: { '@type': 'Organization', name: opts.team },
		about: prettyLeague(opts.league),
		isPartOf: { '@type': 'WebSite', name: 'TDPSearch', url: 'https://tdpsearch.com' }
	});
}
