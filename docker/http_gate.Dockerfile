#== First stage: this is the build stage for Substrate. Here we create the binary.
FROM eightfish-m2-build as builder

#== Second stage: 
FROM docker.io/library/ubuntu:20.04
LABEL description="EightFish:http_gate"

WORKDIR /eightfish

RUN mkdir -p /eightfish/target/wasm32-wasi/release/

COPY --from=builder /usr/local/bin/spin /usr/local/bin
COPY --from=builder /eightfish/http_gate/spin.toml /eightfish/http_gate_spin.toml
COPY --from=builder /eightfish/http_gate/target/wasm32-wasi/release/http_gate.wasm /eightfish/target/wasm32-wasi/release/
