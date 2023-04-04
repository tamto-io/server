DOCKER_IMAGE = "tamto"

build-docker-node:
	DOCKER_BUILDKIT=1 docker build -t $(DOCKER_IMAGE)-node --target node .

build-docker: build-docker-node

run-docker:
	docker compose up -d
