# Message Queue Proof-of-Concept
In this PoC, we are going to test:
1. All APIs provided for shared queue by Envoy proxy-wasm using Rust SDK.
2. Stress test these APIs (especially enqueue and on_queue_ready) under varous loads
# Benchmark
## Enqueue API Stress Test
This test was done to measure latency profile of enqueue_shared_queue() under various loads while consuming messages from the singleston service at the highest rate possible using on_queue_ready().

### Average single enqueue time vs Request load (per second)
![Average single enqueue time vs request count](./bench-data/img/average_enqueue.png)

### Total enqueue time for various loads (per second)
![Average single enqueue time vs request count](./bench-data/img/total_enqueue.png)

On my machine (2 Cores and 4 Logical processors), proxy send empty responses after 680~ requests per second.