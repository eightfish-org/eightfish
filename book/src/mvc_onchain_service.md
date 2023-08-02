# On-chain Service

EightFish is used to develop the decentralized application, so there are some important differences compared to traditional web framework.

For example, in EightFish app, we couldn't use the random function from your local environment, we MUST use the random data from the on-chain runtime. This will affect the generation of all models' id. 

## Random string

EightFish offers a random string source out of box, this random string will change at every request. You can get the inner on-chain random string by:

```
let id = req
    .ext()
    .get("random_str")
    .ok_or(anyhow!("id error"))?
    .to_owned();
```

Please refer [Simple Example](https://github.com/eightfish-org/ef_example_simple_standalone/blob/master/src/article.rs#L63) to get the context of above code.

## Timestamp

The timestamp has the same case with random string, we should use on-chain timestamp rather than local timestamp facility.

You can similarily retrieve the on-chain time by:

```
let time = req
    .ext()
    .get("time")
    .ok_or(anyhow!("generate time failed"))?
    .parse::<i64>()?;
```

The unit of the on-chain timestamp is micro second.


## Others

Generally speaking, you can't do any actions which would violate the state consensus with other EightFish nodes.





