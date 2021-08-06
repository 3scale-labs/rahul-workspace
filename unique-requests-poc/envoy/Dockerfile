FROM envoyproxy/envoy-dev:latest

COPY ./envoy.yaml /etc/envoy/envoy.yaml
COPY ./unique_filter.wasm /usr/local/bin/unique_filter.wasm
RUN chmod go+r /etc/envoy/envoy.yaml /usr/local/bin/unique_filter.wasm
CMD /usr/local/bin/envoy -c /etc/envoy/envoy.yaml -l trace