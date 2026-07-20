import { building } from '$app/environment';
import { env } from '$env/dynamic/public';

// At build time (prerendering) we must call a live Axum by absolute URL,
// because /api and /tdps are not SvelteKit routes. At runtime in the
// browser we use same-origin relative paths (Axum serves the static build).
export const API_ORIGIN: string = building ? (env.PUBLIC_API_ORIGIN ?? '') : '';
