#!/usr/bin/env sh
# Prerender the SvelteKit frontend against a live Axum, then place the output
# where the web server serves it. Requires the API to be reachable.
#   API_ORIGIN   default http://localhost:50000
#   OUT_DIR      default (none) — if set, rsync build output there
set -eu

API_ORIGIN="${API_ORIGIN:-http://localhost:50000}"

echo "prerender: waiting for API at ${API_ORIGIN}/api/papers ..."
i=0
until curl -sf "${API_ORIGIN}/api/papers" >/dev/null 2>&1; do
  i=$((i + 1))
  [ "$i" -gt 60 ] && { echo "prerender: API not reachable after 60s" >&2; exit 1; }
  sleep 2
done

echo "prerender: building frontend ..."
cd "$(dirname "$0")/../frontend"
# Deps are installed at image-build/host time (via Nexus). Only install if absent
# — never repoint the registry.
[ -d node_modules ] || npm ci
PUBLIC_API_ORIGIN="${API_ORIGIN}" npm run build

if [ -n "${OUT_DIR:-}" ]; then
  echo "prerender: syncing build -> ${OUT_DIR}"
  mkdir -p "${OUT_DIR}"
  # Clear all existing contents including dotfiles (a plain `rm -rf DIR/*` glob
  # skips hidden entries), without removing the mount point itself.
  find "${OUT_DIR:?}" -mindepth 1 -delete 2>/dev/null || true
  cp -r build/. "${OUT_DIR}/"
fi
echo "prerender: done"
