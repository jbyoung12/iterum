.PHONY: build test run redis

build:
	cargo build --release

redis:
	docker compose up -d redis

run:
	cargo run

test:
	cargo test
