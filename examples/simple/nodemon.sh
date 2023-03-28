nodemon --watch target/wasm32-wasi/release/simple.wasm --ext .wasm --verbose --legacy-watch --signal SIGINT --exec 'spin up --file spin.toml'
