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
                if ef_res.is_err() {
                    return Err(anyhow!());
                }

                // we check the intermediate result  in the framework internal 
                let pair_list = inner_stuffs_on_query_result(&ef_res);

                // ef_res.info here could also contain the modelname 
                // let modelname = ef_res.info;

                // we can retrieve the model name from the path
                // but that will force the developer use a strict unified url shcema in his product
                // the names in query and post must MATCH
                tail_query_process(&redis_addr, reqid, &modelname, &pair_list);
            }
            "post" => {
                let method = Method::Post;
                let path = msg_obj.model.to_owned();
                let modelname = retrieve_model_name(&path);
                let payload: Payload = serde_json::from_slice(&msg_obj.data)?;

                let ef_req = EightFishRequest::new(method, path, payload.reqdata);

                let ef_res = self.app.handle(ef_req);

                let pair_list = inner_stuffs_on_post_result(&ef_res, &instance_hash);

                tail_post_process(&redis_addr, reqid, &modelname, &pair_list);
            }
            "update_index" => {
                // handle the result of the update_index call event
                // the format of the msg_obj.data is: reqid:id:hash
                // and msg.model is model, msg.action is action
                let reqid = ...;
                let id = ...;
                let hash = ...;
                
                let _ = redis::set(&redis_addr, "cache:status:{reqid}", b"true");
                let _ = redis::set(&redis_addr, "cache:{reqid}", &(id.to_string() + ":" + hash).as_bytes()); 

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



fn tail_query_process(redis_addr: &str, reqid: &str, modelname: &str, pair_list: &Vec<(String, String)>) {
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

fn tail_post_process(redis_addr: &str, reqid: &str, modelname: &str, pair_list: &Vec<(String, String)>) {
    let payload = json!({
        "reqid": reqid,
        "reqdata": pair_list,
    });

    let json_to_send = json!({
        "model": modelname,
        "action": "update_index",
        "data": payload.to_string().as_bytes().to_vec(),
        "time": 0
    });
    _ = redis::publish(&redis_addr, "spin2proxy", &json_to_send.to_string().as_bytes());

}



/// for performance
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArticleHash {
    item: Article,
    hash: String,
}


trait IdHashPair {
    ///
    fn id(&self) -> String;

    ///
    fn hash(&self) -> String;

}

impl IdHashPair for ArticleHash {
    
    fn id(&self) -> String {
        self.item.id.to_string()
    }

    fn hash(&self) -> String {
        self.hash.to_string()
    }
}

// TODO: fill all logic
fn inner_stuffs_on_query_result(res: &EightFishResponse) -> Result<Vec<String, String>, > {
    let table_name = res.info;
    let results = &res.results;
    // get the id list from obj list
    let ids = ...;
    let ids_string = ids. to a list, delimited by a comma;

    let query_string = format!("select id, hash from {table_name}_idhash where id in ({ids_string})");
    let rowset = pg::query(&pg_addr, &query_string, &vec![]).unwrap();

    // convert the raw vec[u8] to every rust struct filed, and convert the whole into a
    // rust struct vec, later we may find a gerneral type converter way
    let mut idhash_map: HashMap<String, String> = HashMap::New();
    for row in rowset.rows {
        let id = String::decode(&row[0])?;
        let hash = String::decode(&row[1])?;

        idhash_map.insert(id, hash);
    }

    // iterate on the input results to check
    // meanwhile construct a (id-hash) pair list
    let mut pair_list: Vec<(String, String)> = vec![];
    for obj in results {
        let id = obj.id();
        let calced_hash = obj.calc_hash().unwrap();
        let hash_from_map = idhash_map.get(id).expect("");
        if calced_hash != hash_from_map {
            return Err(anyhow!("Hash mismatching.".to_string()));
        }
        pair_list.push((id, hash_from_map));
    }

    // store to cache for http gate to retrieve
    let data_to_cache = serde_json::to_string(results).unwrap();
    _ = redis::set(redis_addr, &format!("tmp:cache:{reqid}"), &data_to_cache.as_bytes());

    Ok(pair_list)
}

fn inner_stuffs_on_post_result(res: &EightFishResponse) -> Result<Vec<String, String>, > {
    let info_items = res.info. ...;
    let table_name = info_items[0];

    if res.results.is_empty() && info_items[1] == "delete" {
        // means it's a delete action
        id = info_items[2];
        ins_hash = "".to_string();
    } else {
        let obj = &res.results[0];
        let id = obj.id();
        ins_hash = obj.calc_hash();
    }

    if info_items[1] == "delete" {
        let sql_string = format!("delete {table_name}_idhash where id='{id}'");
        let _execute_results = pg::execute(&pg_addr, &sql_string, &vec![]);
        // TODO: check the pg result

    } else if info_items[1] == "update" {
        let sql_string = format!("update {table_name}_idhash set id={id}, hash={ins_hash} where id='{id}'");
        let _execute_results = pg::execute(&pg_addr, &sql_string, &vec![]);

    } else {
        let sql_string = format!("insert into {table_name}_idhash values ({}, {})", id, ins_hash);
        let _execute_results = pg::execute(&pg_addr, &sql_string, &vec![]);
        // TODO: check the pg result

    }
    // TODO: handle update

    let mut pair_list: Vec<(String, String)> = vec![];
    pair_list.push((id, ins_hash));

    Ok(pair_list)
}


fn calc_hash<T: Serialize>(obj: &T) -> Result<String> {
    // I think we can use json_digest to do the deterministic hash calculating
    // https://docs.rs/json-digest/0.0.16/json_digest/
    let json_val= serde_json::to_value(obj).unwrap();
    let digest = json_digest::digest_data(&json_val).unwrap();

    Ok(digest)
}


