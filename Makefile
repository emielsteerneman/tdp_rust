.PHONY: qdrant-restart web ui docker docker-logs docker-down init clean
.PHONY: activity repl search search-text search-table search-image create-idf mcp leagues

# --- Services ---

web:
	cargo run -p web

mcp:
	cargo run -p mcp

ui:
	cd frontend && npm run dev -- --host --port 50080

# --- Infrastructure ---

qdrant-restart:
	docker stop qdrant || true
	docker rm qdrant || true
	docker run --network=host --name qdrant -d qdrant/qdrant:v1.16

qdrant-snapshot:
	./scripts/qdrant_create_download_delete_snapshot.sh qdrant.snapshot

docker:
	docker compose up --build

docker-logs:
	docker compose logs -f

docker-down:
	docker compose down

# --- Tools ---

init:
	cargo run --release -p tools --bin initialize

create-idf:
	cargo run -p tools --bin create_idf

repl:
	cargo run -p tools --bin repl

search:
	cargo run -p tools --bin search_by_sentence -- $(filter-out $@,$(MAKECMDGOALS))

search-text:
	cargo run -p tools --bin search_by_sentence -- $(filter-out $@,$(MAKECMDGOALS)) --type text

search-table:
	cargo run -p tools --bin search_by_sentence -- $(filter-out $@,$(MAKECMDGOALS)) --type table

search-image:
	cargo run -p tools --bin search_by_sentence -- $(filter-out $@,$(MAKECMDGOALS)) --type image

activity:
	cargo run -p tools --bin activity -- $(ARGS)

# --- Utilities ---

clean: qdrant-restart
	rm -rf data && mkdir data

leagues:
	@curl -s http://localhost:50000/api/leagues | python3 -m json.tool

# Catch-all to allow passing bare arguments to targets like `make search omniwheels`
%:
	@:
