## Flow Test

This document describes how to do post and query flow tests.

First you need to build the docker from the Dockerfile by `./build_docker.sh`.

After it is built successfully, please make sure you have entered the docker container by executing `./run_docker.sh`.

### 1. check the db

Please check the services of redis and postgresql are running. If not, execute:

```
> redis-server /etc/redis/redis.conf
```
and
```
> service postgresql start
```

### 2. run services

You might use tools of screen or tmux to open multiple windows.

There are 4 services should be booted up.

```
# open a new terminal tab:
cd subnode && cargo build --release && target/release/eightfish-subnode --dev

# open a new terminal tab:
cd subxtproxy && cargo build --release && target/release/subxtproxy

# open a new terminal tab:
cd http_gate && spin build --up --follow-all

# open a new terminal tab:
cd examples/simple && spin build --up --follow-all

```

### 3. make http requests

open a new terminal tab:

```
cd examples/simple/flow_tests

# create new artile row
hurl new_article.hurl

# it returns something like:
# {"result":"Ok","id":"5wzxHoJnQd5QhbGcdKkesGiEwtUkynPY4JFrUrm9Us5q"}

# copy the returned id and paste to the right place of the next command line to get this article
hurl --variable id=5wzxHoJnQd5QhbGcdKkesGiEwtUkynPY4JFrUrm9Us5q get_one_article.hurl

# it returns something like:
# [{"id":"5wzxHoJnQd5QhbGcdKkesGiEwtUkynPY4JFrUrm9Us5q","title":"test111","content":"this is the content of test111","authorname":"mike tang"}]

```

Congratulations to you! You have gone through the first EightFlow app.


