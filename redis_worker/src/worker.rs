#![allow(dead_code)]
use std::collections::HashMap;
use anyhow::{Result, anyhow, bail};
use bytes::Bytes;
use spin_sdk::{
    redis, pg,
};
use serde::{Serialize, Deserialize};
use serde_json::json;
use eightfish::{Method};

const REDIS_URL_ENV: &str = "REDIS_URL";
const DB_URL_ENV: &str = "DB_URL";
const TMP_CACHE_RESULTS: &str = "tmp:cache:{}";
const CACHE_STATUS_RESULTS: &str = "cache:status:{}";
const CACHE_RESULTS: &str = "cache:{}";
const CHANNEL_SPIN2PROXY: &str = "spin2proxy";


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
                // path info put in the model field from the http_gate
                let path = msg_obj.model.to_owned();
                let payload: Payload = serde_json::from_slice(&msg_obj.data)?;
                let reqid = payload.reqid.to_owned();
                let reqdata = payload.reqdata.to_owned();

                let ef_req = EightFishRequest::new(method, path, reqdata);

                let ef_res = self.app.handle(ef_req);
                if ef_res.is_err() {
                    return Err(anyhow!("fooo get"));
                }
                let ef_res = ef_res.unwrap();

                // we check the intermediate result  in the framework internal 
                let pair_list = inner_stuffs_on_query_result(&reqid, &ef_res).unwrap();

                let redis_addr = std::env::var(REDIS_URL_ENV)?;
                let modelname = ef_res.info.model_name.to_owned();
                // we can retrieve the model name from the path
                // but that will force the developer use a strict unified url shcema in his product
                // the names in query and post must MATCH
                tail_query_process(&redis_addr, &reqid, &modelname, &pair_list);
            }
            "post" => {
                let method = Method::Post;
                let path = msg_obj.model.to_owned();
                let payload: Payload = serde_json::from_slice(&msg_obj.data)?;
                let reqid = payload.reqid.to_owned();
                let reqdata = payload.reqdata.to_owned();

                let ef_req = EightFishRequest::new(method, path, reqdata);

                let ef_res = self.app.handle(ef_req);
                if ef_res.is_err() {
                    return Err(anyhow!("fooo post"));
                }
                let ef_res = ef_res.unwrap();

                let pair_list = inner_stuffs_on_post_result(&ef_res).unwrap();

                tail_post_process(&redis_addr, &reqid, &modelname, &pair_list);
            }
            "update_index" => {
                // Callback: handle the result of the update_index call event
                // the format of the msg_obj.data is: reqid:id:hash
                // and msg.model is model, msg.action is action
                let v: Vec<&str> = str::from_utf8(&msg_obj.data).unwrap().split(':').collect();
                let reqid = &v[0];
                let id = &v[1];
                let hash = &v[2];
                
                // while getting the index updated callback, we put result http_gate wants into redis
                // cache
                let redis_addr = std::env::var(REDIS_URL_ENV)?;
                let cache_key = format!(CACHE_STATUS_RESULTS, reqid);
                _ = redis::set(&redis_addr, cache_key, b"true");
                let cache_key = format!(CACHE_RESULTS, reqid);
                _ = redis::set(&redis_addr, cache_key, &(id.to_string() + ":" + hash).as_bytes()); 

            }
            "check_pair_list" => {
                let redis_addr = std::env::var(REDIS_URL_ENV)?;

                // handle the result of the check_pair_list
                let payload: Payload = serde_json::from_slice(&msg_obj.data)?;
                let reqid = payload.reqid.clone();
                let reqdata = payload.reqdata.clone().unwrap();

                if &reqdata == "true" {
                    // check pass, get content from the tmp cache and write this content to a cache
                    let tmpdata = redis::get(&redis_addr, &format!(TMP_CACHE_RESULTS, reqid));
                    _ = redis::set(&redis_addr, &format!(CACHE_STATUS_RESULTS, reqid), b"true");
                    if tmpdata.is_ok() {
                        let _ = redis::set(&redis_addr, &format!(CACHE_RESULTS, reqid), &tmpdata.unwrap());
                    }
                    // delete the tmp cache
                    _ = redis::del(&redis_addr, &format!(TMP_CACHE_RESULTS, reqid));
                }
                else {
                    _ = redis::set(&redis_addr, &format!(CACHE_STATUS_RESULTS, reqid), b"false");
                    let data = "check of pair list wrong!";
                    _ = redis::set(&redis_addr, CACHE_RESULTS, data);
                    // clear another tmp cache key
                    _ = redis::del(&redis_addr, &format!(TMP_CACHE_RESULTS, reqid));

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
    _ = redis::publish(&redis_addr, CHANNEL_SPIN2PROXY, &json_to_send.to_string().as_bytes());
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

    _ = redis::publish(&redis_addr, CHANNEL_SPIN2PROXY, &json_to_send.to_string().as_bytes());

}


fn inner_stuffs_on_query_result(reqid: &str, res: &EightFishResponse) -> Result<Vec<String, String>> {
    let table_name = res.info.model_name;
    let pair_list = &res.pair_list.clone().unwrap();
    // get the id list from obj list
    let ids = pair_list.map(|&(id, hash)| id.to_owned()).collect();
    let ids_string = ids.join(',');

    let query_string = format!("select id, hash from {table_name}_idhash where id in ({ids_string})");
    let rowset = pg::query(&pg_addr, &query_string, &vec![]).unwrap();

    let mut idhash_map: HashMap<String, String> = HashMap::New();
    for row in rowset.rows {
        let id = String::decode(&row[0])?;
        let hash = String::decode(&row[1])?;

        idhash_map.insert(id, hash);
    }

    // iterate on the input results to check
    for (id, chash) in pair_list {
        let hash_from_map = idhash_map.get(id).expect("");
        if chash != hash_from_map {
            return Err(anyhow!("Hash mismatching.".to_string()));
        }
    }

    // store to cache for http gate to retrieve
    let data_to_cache = res.results.unwrap_or("".to_string());
    _ = redis::set(redis_addr, &format!(TMP_CACHE_RESULTS, reqid), &data_to_cache.as_bytes());

    Ok(pair_list)
}

fn inner_stuffs_on_post_result(res: &EightFishResponse) -> Result<Vec<String, String>> {
    let table_name = &res.info.model_name;
    //let id = &res.info.target;
    let action = &res.info.action;
    let mut id = String::new();
    let mut ins_hash = String::new();
    
    if res.pair_list.is_some() {
        // here, we just process single updated item returning
        let pair = res.pair_list.clone().unwrap()[0];
        id = pair.0;
        ins_hash = pair.1;
    }
    else {
        return bail!("No pair.".to_string());
    }

    if action == "new"{
        let sql_string = format!("insert into {table_name}_idhash values ({}, {})", id, ins_hash);
        let _execute_results = pg::execute(&pg_addr, &sql_string, &vec![]);
        // TODO: check the pg result

    } else if action == "update" {
        let sql_string = format!("update {table_name}_idhash set hash={ins_hash} where id='{id}'");
        let _execute_results = pg::execute(&pg_addr, &sql_string, &vec![]);

    } else if action == "delete" {
        let sql_string = format!("delete {table_name}_idhash where id='{id}'");
        let _execute_results = pg::execute(&pg_addr, &sql_string, &vec![]);
        // TODO: check the pg result
    }
    else {

    }

    let mut pair_list: Vec<(String, String)> = vec![];
    pair_list.push((id, ins_hash));

    Ok(pair_list)
}


