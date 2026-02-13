.PHONY: qdrant-restart web ui docker docker-logs docker-down init clean
.PHONY: activity repl search-by-sentence create-idf mcp leagues

# --- Services ---

web:
	cargo run -p web

mcp:
	cargo run -p mcp

ui:
	cd frontend && npm run dev -- --host --port 8003

# --- Infrastructure ---

qdrant-restart:
	docker stop qdrant || true
	docker rm qdrant || true
	docker run --network=host --name qdrant -d qdrant/qdrant:v1.16

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

search-by-sentence:
	cargo run -p tools --bin search_by_sentence

activity:
	cargo run -p tools --bin activity -- $(ARGS)

# --- Utilities ---

clean: qdrant-restart
	rm -f my_sqlite.db my_sqlite.db-shm my_sqlite.db-wal

leagues:
	@curl -s http://localhost:8081/api/leagues | python3 -m json.tool
