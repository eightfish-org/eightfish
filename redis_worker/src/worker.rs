#![allow(dead_code)]
use std::collections::HashMap;
use anyhow::{Result, anyhow};
use bytes::Bytes;
use spin_sdk::{
    redis, pg,
};
use serde::{Serialize, Deserialize};
use serde_json::json;
use uuid::Uuid;

#[derive(Deserialize, Debug)]
pub struct InputOutputObject {
    model: String,
    action: String,
    data: Vec<u8>,
    time: u64,
}

#[derive(Deserialize, Debug)]
pub struct Payload {
    reqid: String,
    reqdata: Option<String>,
}

#[derive(Deserialize, Debug)]
pub struct Payload2 {
    reqid: String,
    reqdata: String,
}

pub struct Worker {
    app: EightFishApp,    
}

impl Worker {
    pub fn mount(app: EightFishApp) -> Self {
        Worker {
            app
        }
    }

    pub fn work(self, message: Bytes) -> Result<()> {

        let msg_obj: InputOutputObject = serde_json::from_slice(&message)?;

        match &msg_obj.action[..] {
            "query" => {
                let method = Method::Get;
                let path = msg_obj.model.to_owned();
                let modelname = retrieve_model_name(&path);
                let payload: Payload = serde_json::from_slice(&msg_obj.data)?;

                let ef_req = EightFishRequest::new(method, path, payload.reqdata);

                let ef_res = self.app.handle(ef_req);

                // we can retrieve the model name from the path
                // but that will force the developer use a strict unified url shcema in his product
                // the names in query and post must MATCH
                tail_query_process(&redis_addr, reqid, &modelname, &ef_res.results);
            }
            "post" => {
                let method = Method::Post;
                let path = msg_obj.model.to_owned();
                let modelname = retrieve_model_name(&path);
                let payload: Payload = serde_json::from_slice(&msg_obj.data)?;

                let ef_req = EightFishRequest::new(method, path, payload.reqdata);

                let ef_res = self.app.handle(ef_req);

                let instance = &ef_res.results[0];
                let instance_id = instance.id();
 
                // the case of 'delete' must be separated from the 'create' and 'update'
                let instance_hash = if ef_res.info == "delete" {
                    // to make the on-chain indexer delete the corresponding item
                    "".to_string();
                } else {
                    instance.hash();
                }

                tail_post_process(&redis_addr, reqid, &modelname, &instance_id, &instance_hash);
            }
            "update_index" => {
                // handle the result of the update_index call event
                // the format of the msg_obj.data is: reqid:id:hash
                // and msg.model is model, msg.action is action
            }
            "check_pair_list" => {
                let redis_addr = std::env::var(REDIS_URL_ENV)?;

                // handle the result of the check_pair_list
                let payload: Payload2 = serde_json::from_slice(&msg_obj.data)?;
                let reqid = payload.reqid.clone();
                let result = &payload.reqdata;

                // put result in path
                if result == "true" {
                    // check pass, write this content to a cache
                    let tmpdata = redis::get(&redis_addr, &format!("tmp:cache:{}", reqid));
                    let _ = redis::set(&redis_addr, &format!("cache:status:{}", reqid), b"true");
                    if tmpdata.is_ok() {
                        let _ = redis::set(&redis_addr, &format!("cache:{}", reqid), &tmpdata.unwrap());
                    }
                    // delete the tmp cache
                    //let _ = redis::del(&redis_addr, "tmp:cache:{reqid}");
                }
                else {
                    // if not true, get which one is not equal, this info is in that data field '(id,
                    // hash) is not right'
                    let _ = redis::set(&redis_addr, &format!("cache:status:{}", reqid), b"false");
                    //let _ = redis::set(&redis_addr, "cache:{reqid}", data);
                    // clear another tmp cache key
                    //let _ = redis::del(&redis_addr, "tmp:cache:{reqid}");

                }
            }
            &_ => {
                todo!()
            }   

        }
    }

}



fn tail_query_process<T: IdHashPair + Serialize>(redis_addr: &str, reqid: &str, modelname: &str, results: &Vec<T>) {
    // write this content to a tmp cache
    let data_to_cache = serde_json::to_string(results).unwrap();
    _ = redis::set(redis_addr, &format!("tmp:cache:{reqid}"), &data_to_cache.as_bytes());

    // construct a (id-hash) pair list
    let pair_list: Vec<(String, String)> = results.iter().map(|obj| (obj.id(), obj.hash())).collect();
    //let pair_list_string = serde_json.to_string(&pair_list).unwrap();
    let payload = json!({
        "reqid": reqid,
        "reqdata": pair_list,
    });
    // XXX: here, maybe it's better to put check_pair_list value to action field
    let json_to_send = json!({
        "model": modelname,
        "action": "check_pair_list",
        "data": payload.to_string().as_bytes().to_vec(),
        "time": 0
    });

    // send this to the redis channel to subxt to query rpc
    _ = redis::publish(&redis_addr, "spin2proxy", &json_to_send.to_string().as_bytes());
}

fn tail_post_process(redis_addr: &str, reqid: &str, modelname: &str, id: &str, hash: &str) {
    // if execute_results is ok
    // send the id hash pair (as a list) to the redis channel
    // construct a (id-hash) pair list
    let pair_list = Vec::new().push((id.to_owned(), hash.to_owned()));
    //let pair_list_string = serde_json.to_string(&pair_list).unwrap();
    let payload = json!({
        "reqid": reqid,
        "reqdata": pair_list,
    });
    // XXX: better to put update_index directive to action field
    let json_to_send = json!({
        "model": modelname,
        "action": "update_index",
        "data": payload.to_string().as_bytes().to_vec(),
        "time": 0
    });
    _ = redis::publish(&redis_addr, "spin2proxy", &json_to_send.to_string().as_bytes());

    // we may wait update id-hash pair successfully, or not
    // here is not, then we won't wait for one more block interval time
    let _ = redis::set(&redis_addr, "cache:status:{reqid}", b"true");
    let _ = redis::set(&redis_addr, "cache:{reqid}", &(id.to_string() + ":" + hash).as_bytes()); // update the id and hash back

}
