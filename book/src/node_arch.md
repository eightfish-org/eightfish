# Arch of the EightFish Node

![](https://raw.githubusercontent.com/eightfish-org/eightfish_assets/master/node_arch.png)

Explanation:

There are some components in an EightFish node.

- subnode: the blockchain node located inside of an EightFish node.
- subxtproxy: the subnode rpc client used to connect the subnode with the spin worker.
- redis: used as msg channels and data caches
- postgres: used as the storage of raw data
- http gate: used as the interface of http data service
- spin worker: acts as the core business engine in EightFish workflow.
- MVC: an easy engineering layer for programmer to write logic, in a style of Web CRUD.
- Tiny ORM: a set of helper ORM utilities to make the interactions with SQL statements easier and safer.

