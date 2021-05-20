FROM envoyproxy/envoy-dev:latest

COPY ./envoy.yaml /etc/envoy/envoy.yaml
COPY ./queue_poc_filter.wasm /usr/local/bin/queue_poc_filter.wasm
RUN chmod go+r /etc/envoy/envoy.yaml /usr/local/bin/queue_poc_filter.wasm
CMD /usr/local/bin/envoy -c /etc/envoy/envoy.yaml -l debug