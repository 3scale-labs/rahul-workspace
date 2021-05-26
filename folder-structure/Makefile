build:
	cargo build --target wasm32-unknown-unknown --release
	cp target/wasm32-unknown-unknown/release/queue_poc_filter.wasm ./envoy/queue_poc_filter.wasm
	cp target/wasm32-unknown-unknown/release/queue_poc_singleton.wasm ./envoy/queue_poc_singleton.wasm
