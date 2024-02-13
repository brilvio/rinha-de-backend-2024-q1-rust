include .env

DOCKER_IMAGE = brilvio/rinha-de-backend-2024-q1-rust

build-docker:
	docker build -t $(DOCKER_IMAGE) .

docker-run:
	docker compose up -d && docker container stats