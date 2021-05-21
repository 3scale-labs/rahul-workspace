build: queue-poc-singleton.wasm

queue-poc-singleton.wasm:
	cargo build --target wasm32-unknown-unknown --release
	cp target/wasm32-unknown-unknown/release/queue_poc_singleton.wasm ../envoy/queue_poc_singleton.wasm

.PHONY: clean
clean:
	rm queue_poc_singleton.wasm || true