.PHONY: qdrant-restart

qdrant-restart:
	docker stop qdrant || true
	docker rm qdrant || true
	docker run --network=host --name qdrant -d qdrant/qdrant:v1.16

init:
	cargo run --release -p pipeline --bin initialize