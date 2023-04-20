#== First stage: this is the build stage for Substrate. Here we create the binary.
FROM docker.io/paritytech/ci-linux:production as builder

WORKDIR /eightfish
COPY . .

# install rust components
RUN rustup target add wasm32-unknown-unknown --toolchain nightly
RUN rustup target add wasm32-unknown-unknown
RUN rustup target add wasm32-wasi

# install third tools
RUN cd /tmp/ && curl -fsSL https://developer.fermyon.com/downloads/install.sh | bash && mv spin /usr/local/bin/
#RUN cargo install subxt-cli && cargo install hurl

RUN cd subnode && cargo build --release
RUN cd subxtproxy && cargo build --release
RUN cd http_gate && spin build
RUN cd examples/simple && spin build 

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
