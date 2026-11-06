.PHONY: build run fmt clippy test docker
build:
	cargo build --release
run:
	cargo run -- gateway
fmt:
	cargo fmt
clippy:
	cargo clippy --all-targets --all-features -D warnings
test:
	cargo test --test integration
docker:
	docker compose build && docker compose up -d
