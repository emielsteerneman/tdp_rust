/** Per-league-major color tokens used for badges and sidebar indicators. */

export interface LeagueColor {
	badge: string;
	dot: string;
}

const COLORS: Record<string, LeagueColor> = {
	soccer: {
		badge: 'bg-blue-100 text-blue-800 dark:bg-blue-900/50 dark:text-blue-300',
		dot: 'bg-blue-500',
	},
	rescue: {
		badge: 'bg-red-100 text-red-800 dark:bg-red-900/50 dark:text-red-300',
		dot: 'bg-red-500',
	},
	'@home': {
		badge: 'bg-yellow-100 text-yellow-800 dark:bg-yellow-900/50 dark:text-yellow-300',
		dot: 'bg-yellow-500',
	},
	industrial: {
		badge: 'bg-emerald-100 text-emerald-800 dark:bg-emerald-900/50 dark:text-emerald-300',
		dot: 'bg-emerald-500',
	},
};

const FALLBACK: LeagueColor = {
	badge: 'bg-gray-100 text-gray-800 dark:bg-gray-800 dark:text-gray-300',
	dot: 'bg-gray-400',
};

/** Get color tokens for a league by its major category (e.g. "Soccer" or "soccer"). */
export function getLeagueColor(leagueMajor: string): LeagueColor {
	return COLORS[leagueMajor.toLowerCase()] ?? FALLBACK;
}

/** Convenience: get the badge class string for a league major. */
export function getLeagueBadgeColor(leagueMajor: string): string {
	return getLeagueColor(leagueMajor).badge;
}
