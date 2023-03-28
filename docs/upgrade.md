
## Upgrade

This document describes how to use the upgrading feature.

The same as the flow tests, we do some checkes at first.

### 1. check the db

Please check the services of redis and postgresql are running. If not, execute:

```
> redis-server /etc/redis/redis.conf
```
and
```
> service postgresql start
```

### 2. check services

You might use tools of screen or tmux to open multiple windows.

There are 3 services should be booted up.

```
# open a new terminal tab:
cd subnode && cargo build --release && target/release/eightfish-subnode --dev

# open a new terminal tab:
cd subxtproxy && cargo build --release && target/release/subxtproxy

# open a new terminal tab:
cd http_gate && spin up


```

### 3. boot the application

Instead of using `spin up` to boot the `simple` application directly, now we use `nodemon` to monitoringly 
boot it.

````
# open a new terminal tab:
cd examples/simple && ./nodemon.sh
````
Ref: 

![](https://raw.githubusercontent.com/eightfish-org/eightfish_assets/master/s1.png)


### 3.1. test the version

```
cd examples/simple/flow_tests
hurl version.hurl
```

We will get:

```
{"version": 1}
```


### 4. compile a new version of this application

We copied a project named `simple2` in the examples directory, did some minor modifications on it, change 
the version number result from 1 to 2.

Now let's compile it:

```
cd examples/simple2 && spin build
```

This will generate the compiled wasm file at: examples/simple2/target/wasm32-wasi/release/simple.wasm, latter 
we will upload this wasm to the subnode on-chain storage.

### 5. upload to on-chain

Now enter the `upgrade` subdir of eightfish.

```
cd upgrade
cargo run --bin upload_wasm
```

This step will upload the previous wasm file to on-chain storage, this path is configured at the 
`upgrade/.env`.


Ref: 

![](https://raw.githubusercontent.com/eightfish-org/eightfish_assets/master/s2.png)

### 6. boot the new on-chain appfile daemon

Ordinarily, we should keep this daemon running from the start, but here we just test the upgrade flow, so we put 
it at this step.

```
cd upgrade
cargo run --bin eightfish-upgrade
```

This step will check whether there is new version of wasm file on chain, if yes, download it, if no, sleep for
some seconds, and try to check it again.

Ref: 

![](https://raw.githubusercontent.com/eightfish-org/eightfish_assets/master/s3.png)

This daemon will download the on-chain wasm file blob to the place where `nodemon` monitors, so once the file 
saved onto local disk and changes, the `nodemon` will restart the application automaticlally.

Ref: 

![](https://raw.githubusercontent.com/eightfish-org/eightfish_assets/master/s4.png)


### 7. test the version again

```
cd examples/simple/flow_tests
hurl version.hurl
```

We will get:

```
{"version": 2}
```

As you noticed, the version has been upgrade from 1 to 2. We have done the upgrading process, yes, you can 
get that: we use the blockchain to distribute the software, and execute it off chain. This is an awesome 
feature of EightFish.



