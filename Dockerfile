FROM rust:1.67

WORKDIR /usr/src/eightfish
COPY . .

RUN apt update
RUN apt install -y build-essential git clang curl libssl-dev llvm libudev-dev make protobuf-compiler pkg-config

RUN rustup update nightly
RUN rustup target add wasm32-unknown-unknown --toolchain nightly

RUN curl -fsSL https://developer.fermyon.com/downloads/install.sh | bash && mv spin /usr/local/bin/
RUN rustup target add wasm32-wasi

RUN apt install -y redis-server redis-tools
RUN apt install -y postgresql

RUN cargo install subxt-cli
RUN cargo install hurl

#RUN cd subnode && cargo build --release
#RUN cd subxtproxy && cargo build --release
#RUN cd http_gate && spin build
#RUN cd examples/simple/ && spin build

