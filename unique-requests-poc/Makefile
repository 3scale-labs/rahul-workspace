.PHONY: clean run

build: unique-filter

unique-filter:
	cargo build --target wasm32-unknown-unknown --release --manifest-path=./filter/Cargo.toml
	cp filter/target/wasm32-unknown-unknown/release/unique_filter.wasm ./envoy/unique_filter.wasm

run:
	@echo "> Starting services"
	docker-compose -f envoy/docker-compose.yaml up --build

clean:
	rm unique-filter || true