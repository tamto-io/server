DOCKER_IMAGE = "tamto"

build-docker-node:
	DOCKER_BUILDKIT=1 docker build -t $(DOCKER_IMAGE)-node --target node .

build-docker: build-docker-node

run-docker:
	docker compose up -d

build:
	cargo build --release

run : build
	./target/release/server --listen "[::1]:42000"

run-local: build
	./scripts/run-nodes.sh -n 10 -p 42050
