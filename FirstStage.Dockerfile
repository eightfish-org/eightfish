#== First stage: this is the build stage for Substrate. Here we create the binary.
FROM docker.io/paritytech/ci-linux:1.68.2-bullseye as builder

WORKDIR /eightfish
COPY . .

# install rust components
RUN rustup target add wasm32-unknown-unknown --toolchain nightly
RUN rustup target add wasm32-wasi

# install third tools
RUN cd /tmp/ && curl -fsSL https://developer.fermyon.com/downloads/install.sh | bash && mv spin /usr/local/bin/

RUN cd subnode && cargo build --release
RUN cd subxtproxy && cargo build --release
RUN cd http_gate && spin build
RUN cd examples/simple && spin build 
