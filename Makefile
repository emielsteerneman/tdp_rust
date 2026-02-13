.PHONY: qdrant-restart
.PHONY: web

qdrant-restart:
	docker stop qdrant || true
	docker rm qdrant || true
	docker run --network=host --name qdrant -d qdrant/qdrant:v1.16

init:
	cargo run --release -p tools --bin initialize

clean: qdrant-restart
	rm my_sqlite.db
	rm my_sqlite.db-shm
	rm my_sqlite.db-wal

web:
	cargo run -p web

ui:
	cd frontend && npm run dev -- --host --port 8003

docker:
	docker compose up --build

docker-logs:
	docker compose logs -f

docker-down:
	docker compose down

activity:
	cargo run -p tools --bin activity -- $(ARGS)

leagues:
	@curl -s http://localhost:8081/api/leagues | python3 -m json.tool
