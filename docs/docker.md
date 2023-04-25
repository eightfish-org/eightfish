
## How to use docker to build and run EightFish tests

### First Step: build the `build` image

Go in the EightFish project root directory, run:

```
./build_docker.sh
```

This will use the FirstStage.Dockerfile to build a compiled image for future use.

### Second Step: build service images

Go to docker/ subdir, execute:

```
cd docker/
./build_services.sh
```

This script will build other 7 images.

### Third step: run docker-compose 

We provide three docker-compose config files, you can choose one to test:

```
docker-compose-1node.yml: to boot a dev node to test
docker-compose-2node.yml: to boot a 2-nodes network to test
docker-compose-4node.yml: to boot a 4-nodes network to test
```

You can run one of them by, e.g:

```
cd docker/
docker compose -f docker-compose-1node.yml up
```

Then you will see the logs outputed in the terminal.

For 1-node testing, the HTTP listening port is 3000, which has been exported to host machine.

For 2-nodes testing, the exported port is: 3000, 3001.

For 4-nodes testing, the exported port is: 3000, 3001, 3002, 3003.

### Fourth step: tests

The testing flow is as same as Milestone I.

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

Congratulations to you! You have gone through the EightFlow tests.

In order to use Polkadot.js app to test, the exported ports are:

- 1-node: 9933, 9944, 30333
- 2-nodes: 9933, 9944, 30333; 9934, 9945, 30334
- 4-nodes: 9933, 9944, 30333; 9934, 9945, 30334; 9935, 9946, 30335; 9936, 9947, 30336
