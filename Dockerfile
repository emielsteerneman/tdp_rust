# Multi-stage Dockerfile for TDP Rust project
# Builds: MCP server, Web/REST API server, SvelteKit frontend, Qdrant with snapshot

# ============================================================================
# Stage 1: Builder - Install all build dependencies
# ============================================================================
FROM rust:latest AS builder

# Install Node.js and build tools
RUN curl -fsSL https://deb.nodesource.com/setup_22.x | bash - && \
    apt-get install -y --no-install-recommends \
    nodejs \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /build

# Copy Cargo workspace and lock file
COPY Cargo.toml Cargo.lock ./
COPY src ./src
COPY mcp ./mcp
COPY web ./web
COPY api ./api
COPY chat ./chat
COPY configuration ./configuration
COPY data_access ./data_access
COPY data_processing ./data_processing
COPY data_structures ./data_structures
COPY tools ./tools

# Copy frontend
COPY frontend ./frontend

# Build Rust binaries in release mode.
# Cache mounts persist the cargo registry and compiled artifacts across builds
# so only changed crates are recompiled. Binaries are copied out of the cache
# mount (which isn't part of the image layer) to a plain path.
RUN --mount=type=cache,target=/usr/local/cargo/registry \
    --mount=type=cache,target=/build/target \
    cargo build --release -p mcp && \
    cargo build --release -p web && \
    cp target/release/mcp /mcp-bin && \
    cp target/release/web /web-bin

# Build frontend
RUN cd frontend && npm ci && npm run build

# ============================================================================
# Stage 2: MCP Server
# ============================================================================
FROM debian:trixie-slim AS mcp

# Install runtime dependencies
RUN apt-get update && apt-get install -y --no-install-recommends \
    ca-certificates \
    libssl3 \
    && rm -rf /var/lib/apt/lists/*

COPY --from=builder /mcp-bin /app/mcp
COPY config.docker.toml /app/config.toml
COPY my_sqlite.db /app/my_sqlite.db
COPY activity.db /app/activity.db

WORKDIR /app

EXPOSE 50001 50002

CMD ["/app/mcp"]

# ============================================================================
# Stage 3: Web/REST API Server (with frontend static files)
# ============================================================================
FROM debian:trixie-slim AS web

# Install runtime dependencies
RUN apt-get update && apt-get install -y --no-install-recommends \
    ca-certificates \
    libssl3 \
    && rm -rf /var/lib/apt/lists/*

COPY --from=builder /web-bin /app/web
COPY config.docker.toml /app/config.toml
COPY my_sqlite.db /app/my_sqlite.db
COPY activity.db /app/activity.db

# Copy frontend static files
COPY --from=builder /build/frontend/build /app/static

WORKDIR /app

EXPOSE 50000

CMD ["/app/web"]

# ============================================================================
# Stage 4: Qdrant with optional snapshot
# ============================================================================
FROM qdrant/qdrant:v1.16 AS qdrant

# Install curl and jq for snapshot restoration
USER root
RUN apt-get update && apt-get install -y --no-install-recommends \
    curl \
    jq \
    && rm -rf /var/lib/apt/lists/*

EXPOSE 6333 6334

# Copy snapshot and restore script
COPY qdrant.snapshot /tmp/qdrant.snapshot
COPY <<'EOF' /docker-entrypoint.d/restore-snapshot.sh
#!/bin/bash
set -e

# Start Qdrant in background
/qdrant/qdrant &
QDRANT_PID=$!

echo "Waiting for Qdrant to start..."
until curl -f http://localhost:6333/healthz >/dev/null 2>&1; do
    sleep 1
done

# Check if collection exists and has data
COLLECTION_EXISTS=$(curl -s http://localhost:6333/collections/chunk/exists | jq -r '.result.exists')
if [ "$COLLECTION_EXISTS" = "true" ]; then
    POINTS_COUNT=$(curl -s http://localhost:6333/collections/chunk | jq -r '.result.points_count')
    if [ "$POINTS_COUNT" -gt 0 ]; then
        echo "Collection 'chunk' already has $POINTS_COUNT points, skipping snapshot restore"
    else
        echo "Collection 'chunk' exists but is empty, deleting and restoring from snapshot..."
        curl -X DELETE http://localhost:6333/collections/chunk
        sleep 2
        echo "Restoring snapshot..."
        curl -X POST 'http://localhost:6333/collections/chunk/snapshots/upload?wait=true' \
            -H 'Content-Type:multipart/form-data' \
            -F 'snapshot=@/tmp/qdrant.snapshot'
        echo "Snapshot restored successfully"
        rm -f /tmp/qdrant.snapshot
    fi
else
    echo "Collection 'chunk' does not exist, restoring from snapshot..."
    curl -X POST 'http://localhost:6333/collections/chunk/snapshots/upload?wait=true' \
        -H 'Content-Type:multipart/form-data' \
        -F 'snapshot=@/tmp/qdrant.snapshot'
    echo "Snapshot restored successfully"
    rm -f /tmp/qdrant.snapshot
fi

# Bring Qdrant to foreground
wait $QDRANT_PID
EOF

RUN chmod +x /docker-entrypoint.d/restore-snapshot.sh

CMD ["/bin/sh", "/docker-entrypoint.d/restore-snapshot.sh"]
