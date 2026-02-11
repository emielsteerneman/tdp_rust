# TDP Rust Frontend

SvelteKit 5 frontend for the TDP Rust project.

## Setup

```bash
npm install
```

## Development

```bash
npm run dev
```

This will start the dev server on `http://localhost:5173`. The Vite proxy is configured to forward `/api/*` requests to the backend at `http://localhost:8080`.

## Building

```bash
npm run build
```

## Tech Stack

- SvelteKit 5
- TypeScript
- Tailwind CSS v4
- @tailwindcss/typography
- marked (Markdown parser)

## API Integration

The frontend communicates with the Axum backend via REST API. See `src/lib/api.ts` for all available endpoints.
