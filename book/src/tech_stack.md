# Tech Stack

EightFish is written in Rust overall. Thanks to the following technologies:

- [Substrate](docs.substrate.io/). The subnode component is built using Substrate, the most powerful blockchain framework made by Parity.
- [Subxt](https://docs.rs/subxt/latest/subxt/). The subxtproxy is built using Subxt, the client RPC library of Substrate.
- [Spin framework](https://www.fermyon.com/spin). An innovative webassembly framework for the future of the cloud.
- redis. No explanation.
- postgres db. No explanation.

## Substrate

The role of Substrate in EightFish.

1. Used to record the incoming raw writing requests, bake them into blocks, like a log system
2. Used to sync runtime state among all EightFish nodes (Substrate network), further to coordinate the state of the SQL db among all nodes
3. Used to store the version of wasm code and make the code of spin worker upgrade forklessly 
4. (not sure) Used to interoperate with other Substrate-based chains by leveraging existing pallets

There is a great [slide](https://docs.google.com/presentation/d/1YIz5rv2R-P8gF-4vvAyHYb0rCGUrUUM0HEpB8ywIcfo/edit?usp=sharing) to explain EightFish for Substrate developers.

## All in Wasm

Almost all EightFish components are compiled into webassembly to run, except for redis and postgres.

We believe that the wasm bytecode will be the standard format of software distribution in the future.