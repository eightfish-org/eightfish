# EightFish

What is EightFish?

EightFish is a development framework (maybe the first one) for the Open Data Application (ODA). The theory of the ODA locates [here](https://medium.com/@daogangtang/the-road-to-open-web-b684879a5571). In short description: EightFish powers the ODAs, ODAs constitute the Open Web.

Concretely, EightFish(8fish) is a framework to develop a decentralized application in Web2 development style.

Unlike the smart contract blockchain tech stack most DApps adopt, EightFish makes your own network, a sovereign network which doesn't reply on any other Web3 layers or services.

By some elaberate designs, EightFish reaches the experiences of Web2/Internet web development, but for OpenWeb/Web3 decentralized application.

Currently this project is under heavily coded, the status is before alpha-releasing.

Note: EightFish itself is not a service/platform/serverless, it is just a dev framework tool.


## Docker

Build the docker devlelopment environment.
```
./build_docker.sh
```

Enter the docker container.

```
./run_docker.sh
```

now you're in the docker dev environment.

## Test

### 1. create postgres db and table

Use psql to create pg db and tables.

```
> su postgres && psql
> alter user postgres with password '123456';
> create database spin_dev;
> \c spin_dev;
> CREATE TABLE article (
        id varchar PRIMARY KEY, 
        title varchar(80) NOT NULL,
        content text NOT NULL,
        authorname varchar(40) NOT NULL
);
> CREATE TABLE article_idhash (
        id varchar PRIMARY KEY,
        hash varchar NOT NULL
);

```
please use psql to check the results of above actions.


### 2. run services 

You may use tools of screen or tmux to open multiple windows.

```
# open a new terminal tab:
cd subnode && cargo build --release && target/release/eightfish_subnode --dev

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

# copy the returned id and paste to the next command line to get this article 
hurl --variable id=5wzxHoJnQd5QhbGcdKkesGiEwtUkynPY4JFrUrm9Us5q get_one_article.hurl

# it returns something like:
# [{"id":"5wzxHoJnQd5QhbGcdKkesGiEwtUkynPY4JFrUrm9Us5q","title":"test111","content":"this is the content of test111","authorname":"mike tang"}]

```

Congratulations to you! You have done the first EightFlow app.


## License

GPLv3.0


