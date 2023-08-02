
# How to Get Started

## Create a new project 

From the template project of ef_example_simple_standalone.

```
git clone https://github.com/eightfish-org/ef_example_simple_standalone
```

Modify this template and git config infomation. 


## Compile this repository

```
cd ef_example_simple_standalone
spin build
```

here we assumed that you have installed the spin binary and the Rust toolchain set. If you didn't, do it by:

```
# install rust at first, and add the following component
rustup target add wasm32-wasi

# download spin v1.3.0
cd /tmp 
curl -fsSL https://developer.fermyon.com/downloads/install.sh | bash -s -- -v v1.3.0
mv /tmp/spin ~/.cargo/bin/
```

## Copy the spin binary to current directory

```
cp ~/.cargo/bin/spin .
```
## Build the app docker

```
./build_app.sh
```

## Run docker compose.

```
docker compose -f docker-compose-1node.yml up
```

after a while,

## Test

We use `hurl` as the client to do testing. You can install it by:

```
cargo install hurl
```

And then,

```
cd flow_tests

# create new artile row
hurl new_article.hurl

# it returns something like:
# {"result":"Ok","id":"5wzxHoJnQd5QhbGcdKkesGiEwtUkynPY4JFrUrm9Us5q"}

# copy the returned id and paste to the right place of the next command line to get this article
hurl --variable id=5wzxHoJnQd5QhbGcdKkesGiEwtUkynPY4JFrUrm9Us5q get_one_article.hurl

# it returns something like:
# [{"id":"5wzxHoJnQd5QhbGcdKkesGiEwtUkynPY4JFrUrm9Us5q","title":"test111","content":"this is the content of test111","authorname":"mike tang"}]
```
