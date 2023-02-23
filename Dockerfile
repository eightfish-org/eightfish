FROM rust:1.67

WORKDIR /root/eightfish
COPY . .

RUN apt update
RUN apt install -y sudo build-essential git clang curl libssl-dev llvm libudev-dev make protobuf-compiler pkg-config

RUN rustup update nightly
RUN rustup target add wasm32-unknown-unknown --toolchain nightly
RUN rustup target add wasm32-wasi

RUN curl -fsSL https://developer.fermyon.com/downloads/install.sh | bash && mv spin /usr/local/bin/
RUN cargo install subxt-cli && cargo install hurl

RUN apt install -y redis-server redis-tools
RUN apt install -y postgresql && service postgresql start && ./init_pg.sh

EXPOSE 9933
EXPOSE 9944
EXPOSE 30333

