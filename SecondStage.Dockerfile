#== First stage: this is the build stage for Substrate. Here we create the binary.
FROM eightfish-m2-build as builder

#== Second stage: 
FROM docker.io/library/ubuntu:20.04
LABEL description="EightFish: a platform for Open Web"

WORKDIR /eightfish

RUN mkdir -p /eightfish/target/wasm32-wasi/release/

COPY --from=builder /eightfish/subnode/target/release/eightfish-subnode /usr/local/bin
COPY --from=builder /eightfish/subxtproxy/target/release/subxtproxy /usr/local/bin
COPY --from=builder /usr/local/bin/spin /usr/local/bin
COPY --from=builder /eightfish/http_gate/spin.toml /eightfish/http_gate_spin.toml
COPY --from=builder /eightfish/http_gate/target/wasm32-wasi/release/http_gate.wasm /eightfish/target/wasm32-wasi/release/
COPY --from=builder /eightfish/examples/simple/spin.toml /eightfish/simple_app_spin.toml
COPY --from=builder /eightfish/examples/simple/target/wasm32-wasi/release/simple.wasm /eightfish/target/wasm32-wasi/release/


