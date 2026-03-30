#!/bin/bash
set -e

echo "=== Rebuilding Qdrant index from scratch ==="

echo "Step 1: Stopping Docker Compose services..."
docker compose down

echo "Step 2: Removing old data..."
rm -f data/metadata.db qdrant.snapshot

echo "Step 3: Starting standalone Qdrant..."
make qdrant-restart
sleep 2

echo "Step 4: Running initialize pipeline..."
make init

echo "Step 5: Creating Qdrant snapshot..."
make qdrant-snapshot

echo "Step 6: Stopping standalone Qdrant..."
docker stop qdrant
docker rm qdrant || true

echo "Step 7: Building and starting Docker Compose..."
docker compose up -d --build

echo ""
echo "=== Done. Services are starting up. ==="
echo "Run 'docker compose logs -f' to follow startup."
