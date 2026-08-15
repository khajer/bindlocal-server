BIN := connl-server
IMAGE := bindlocal-server:latest

.PHONY: build run release test fmt clippy docker-build docker-run clean

build:
	cargo build

run:
	cargo run

release:
	cargo build --release

test:
	cargo test

fmt:
	cargo fmt

clippy:
	cargo clippy -- -D warnings

docker-build:
	docker build . -t $(IMAGE)

docker-run:
	docker run -p 8080:8080 -p 9090:9090 $(IMAGE)

clean:
	cargo clean
