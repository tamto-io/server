DOCKER_IMAGE = "tamto"

build-docker-node:
	DOCKER_BUILDKIT=1 docker build -t $(DOCKER_IMAGE)-node --target node .

build-docker-admin:
	DOCKER_BUILDKIT=1 docker build -t $(DOCKER_IMAGE)-admin --target admin .

build-docker: build-docker-node build-docker-admin

run-docker:
	docker compose up -d
