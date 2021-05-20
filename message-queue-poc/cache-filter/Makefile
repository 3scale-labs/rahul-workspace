build: queue-poc-filter.wasm

queue-poc-filter.wasm:
	cargo build --target wasm32-unknown-unknown --release
	cp target/wasm32-unknown-unknown/release/queue_poc_filter.wasm ../envoy/queue_poc_filter.wasm

.PHONY: clean
clean:
	rm queue_poc_filter.wasm || true