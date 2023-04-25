#== First stage: this is the build stage for Substrate. Here we create the binary.
FROM eightfish-m2-build as builder

#== Second stage: 
FROM docker.io/library/ubuntu:20.04
LABEL description="EightFish:simple_app"

WORKDIR /eightfish

RUN mkdir -p /eightfish/target/wasm32-wasi/release/

COPY --from=builder /usr/local/bin/spin /usr/local/bin
COPY --from=builder /eightfish/examples/simple/spin-a.toml /eightfish/simple_app_spin.toml
COPY --from=builder /eightfish/examples/simple/target/wasm32-wasi/release/simple.wasm /eightfish/target/wasm32-wasi/release/

