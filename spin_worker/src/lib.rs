#![allow(dead_code)]
use anyhow::{anyhow, bail, Result};
use bytes::Bytes;
use eightfish::{
    App as EightFishApp, Handler, Method, Request as EightFishRequest,
    Response as EightFishResponse,
};
use serde::Deserialize;
use serde_json::json;
use spin_sdk::{
    pg::{self, Decode},
    redis,
};
use std::collections::HashMap;

const REDIS_URL_ENV: &str = "REDIS_URL";
const DB_URL_ENV: &str = "DB_URL";
const TMP_CACHE_RESULTS: &str = "tmp:cache:#";
const CACHE_STATUS_RESULTS: &str = "cache:status:#";
const CACHE_RESULTS: &str = "cache:#";
const CHANNEL_SPIN2PROXY: &str = "spin2proxy";

#[derive(Deserialize, Debug)]
pub struct InputOutputObject {
    model: String,
    action: String,
    data: Vec<u8>,
    ext: Vec<u8>,
}

#[derive(Deserialize, Debug)]
pub struct Payload {
    reqid: String,
    reqdata: Option<String>,
}

#[derive(Deserialize, Debug)]
pub struct ExtPayload {
    time: u64,
    nonce: u64,
    randomvec: Vec<u8>,
}

pub struct Worker {
    app: EightFishApp,
}

impl Worker {
    pub fn mount(app: EightFishApp) -> Self {
        Worker { app }
    }

    pub fn work(self, message: Bytes) -> Result<()> {
        let msg_obj: InputOutputObject = serde_json::from_slice(&message)?;
        println!("Worker::work: msg_obj: {:?}", msg_obj);

        match &msg_obj.action[..] {
            "query" => {
                let redis_addr = std::env::var(REDIS_URL_ENV)?;
                let method = Method::Get;
                // path info put in the model field from the http_gate
                let path = msg_obj.model.to_owned();
                let payload: Payload = serde_json::from_slice(&msg_obj.data)?;
                let reqid = payload.reqid.to_owned();
                let reqdata = payload.reqdata;

                let mut ef_req = EightFishRequest::new(method, path, reqdata);
                println!("Worker::work: in query branch: ef_req");

                let ef_res = self.app.handle(&mut ef_req);
                if ef_res.is_err() {
                    return Err(anyhow!("fooo get"));
                }
                let ef_res = ef_res.unwrap();
                println!("Worker::work: in query branch: ef_res: {:?}", ef_res);

                // we check the intermediate result  in the framework internal
                let pair_list = inner_stuffs_on_query_result(&reqid, &ef_res).unwrap();
                println!("Worker::work: in query branch: pair_list: {:?}", pair_list);

                if pair_list.is_some() {
                    let modelname = ef_res.info().model_name.to_owned();
                    // we can retrieve the model name from the path
                    // but that will force the developer use a strict unified url shcema in his product
                    // the names in query and post must MATCH
                    tail_query_process(&redis_addr, &reqid, &modelname, &pair_list.unwrap());
                }
            }
            "post" => {
                let redis_addr = std::env::var(REDIS_URL_ENV)?;
                let method = Method::Post;
                let path = msg_obj.model.to_owned();
                let payload: Payload = serde_json::from_slice(&msg_obj.data)?;
                let reqid = payload.reqid.to_owned();
                let reqdata = payload.reqdata;
                let ext: ExtPayload = serde_json::from_slice(&msg_obj.ext)?;

                let mut ef_req = EightFishRequest::new(method, path, reqdata);
                println!("Worker::work: in post branch: ef_req");

                // add time to req.ext
                ef_req
                    .ext_mut()
                    .insert("time".to_string(), ext.time.to_string());
                // add nonce to req.ext
                ef_req
                    .ext_mut()
                    .insert("nonce".to_string(), ext.nonce.to_string());
                // encode the vec<u8> as hex string, each byte to 2 bchar. And add randomvec to req.ext
                //let random_string = hex::encode(&ext.randomvec);
                // encode the vec<u8> as base58 string, and add random_str to req.ext
                let random_string = bs58::encode(&ext.randomvec).into_string();
                ef_req
                    .ext_mut()
                    .insert("random_str".to_string(), random_string);

                let ef_res = self.app.handle(&mut ef_req);
                if ef_res.is_err() {
                    return Err(anyhow!("fooo post"));
                }
                println!("Worker::work: in post branch: ef_res: {:?}", ef_res);
                let ef_res = ef_res.unwrap();

                let pair_list = inner_stuffs_on_post_result(&ef_res).unwrap();
                println!("Worker::work: in post branch: pair_list: {:?}", pair_list);

                let modelname = ef_res.info().model_name.to_owned();

                tail_post_process(&redis_addr, &reqid, &modelname, &pair_list);
            }
            "update_index" => {
                // Callback: handle the result of the update_index call event
                // the format of the msg_obj.data is: reqid:id:hash
                // and msg.model is model, msg.action is action
                //let v: Vec<&str> = std::str::from_utf8(&msg_obj.data).unwrap().split(':').collect();
                //println!("index_update callback: v: {:?}", v);
                //let reqid = &v[0];
                //let id = &v[1];
                //let hash = &v[2];
                let payload: Payload = serde_json::from_slice(&msg_obj.data)?;
                println!("callback: update_index: payload: {:?}", payload);
                let reqid = payload.reqid.to_owned();
                let id = payload.reqdata.unwrap();

                let result = json!({
                    "result": "Ok",
                    "id": id,
                });

                // while getting the index updated callback, we put result http_gate wants into redis
                // cache
                let redis_addr = std::env::var(REDIS_URL_ENV)?;
                let cache_key = CACHE_STATUS_RESULTS.replace('#', &reqid);
                _ = redis::set(&redis_addr, &cache_key, b"true");
                let cache_key = CACHE_RESULTS.replace('#', &reqid);
                _ = redis::set(&redis_addr, &cache_key, result.to_string().as_bytes());
            }
            "check_pair_list" => {
                let redis_addr = std::env::var(REDIS_URL_ENV)?;

                // handle the result of the check_pair_list
                let payload: Payload = serde_json::from_slice(&msg_obj.data)?;
                let reqid = payload.reqid.clone();
                let reqdata = payload.reqdata.unwrap();

                if &reqdata == "true" {
                    // check pass, get content from the tmp cache and write this content to a cache
                    let tmpdata = redis::get(&redis_addr, &TMP_CACHE_RESULTS.replace('#', &reqid));
                    _ = redis::set(
                        &redis_addr,
                        &CACHE_STATUS_RESULTS.replace('#', &reqid),
                        b"true",
                    );
                    if let Ok(tmpdata) = tmpdata {
                        let _ =
                            redis::set(&redis_addr, &CACHE_RESULTS.replace('#', &reqid), &tmpdata);
                    }
                    // delete the tmp cache
                    _ = redis::del(&redis_addr, &[&TMP_CACHE_RESULTS.replace('#', &reqid)]);
                } else {
                    _ = redis::set(
                        &redis_addr,
                        &CACHE_STATUS_RESULTS.replace('#', &reqid),
                        b"false",
                    );
                    let data = "check of pair list wrong!";
                    _ = redis::set(
                        &redis_addr,
                        &CACHE_RESULTS.replace('#', &reqid),
                        data.as_bytes(),
                    );
                    // clear another tmp cache key
                    _ = redis::del(&redis_addr, &[&TMP_CACHE_RESULTS.replace('#', &reqid)]);
                }
            }
            &_ => {
                todo!()
            }
        }

        Ok(())
    }
}

fn tail_query_process(
    redis_addr: &str,
    reqid: &str,
    modelname: &str,
    pair_list: &Vec<(String, String)>,
) {
    let payload = json!({
        "reqid": reqid,
        "reqdata": Some(pair_list),
    });

    println!("tail_query_process: payload: {:?}", payload);
    // XXX: here, maybe it's better to put check_pair_list value to action field
    let json_to_send = json!({
        "model": modelname,
        "action": "check_pair_list",
        "data": payload.to_string().as_bytes().to_vec(),
        "ext": Vec::<u8>::new(),
    });

    // send this to the redis channel to subxt to query rpc
    _ = redis::publish(
        redis_addr,
        CHANNEL_SPIN2PROXY,
        json_to_send.to_string().as_bytes(),
    );
}

fn tail_post_process(
    redis_addr: &str,
    reqid: &str,
    modelname: &str,
    pair_list: &Vec<(String, String)>,
) {
    let payload = json!({
        "reqid": reqid,
        "reqdata": Some(pair_list),
    });
    println!("tail_post_process: payload: {:?}", payload);

    let json_to_send = json!({
        "model": modelname,
        "action": "update_index",
        "data": payload.to_string().as_bytes().to_vec(),
        "ext": Vec::<u8>::new(),
    });

    _ = redis::publish(
        redis_addr,
        CHANNEL_SPIN2PROXY,
        json_to_send.to_string().as_bytes(),
    );
}

fn inner_stuffs_on_query_result(
    reqid: &str,
    res: &EightFishResponse,
) -> Result<Option<Vec<(String, String)>>> {
    let pg_addr = std::env::var(DB_URL_ENV)?;
    let redis_addr = std::env::var(REDIS_URL_ENV)?;

    if res.pair_list().is_some() {
        let table_name = &res.info().model_name;
        let pair_list = res.pair_list().clone().unwrap();
        // get the id list from obj list
        let ids: Vec<String> = pair_list
            .iter()
            .map(|(id, _)| String::new() + "'" + id + "'")
            .collect();
        let ids_string = ids.join(",");

        let query_string =
            format!("select id, hash from {table_name}_idhash where id in ({ids_string})");
        println!("query_string: {:?}", query_string);
        let rowset = pg::query(&pg_addr, &query_string, &[]).unwrap();

        let mut idhash_map: HashMap<String, String> = HashMap::new();
        for row in rowset.rows {
            let id = String::decode(&row[0])?;
            let hash = String::decode(&row[1])?;

            idhash_map.insert(id, hash);
        }

        // iterate on the input results to check
        for (id, chash) in pair_list.iter() {
            let hash_from_map = idhash_map.get(&id[..]).expect("");
            println!("chash, hash_from_map: {:?}, {:?}", chash, hash_from_map);
            if chash != hash_from_map {
                return Err(anyhow!("Hash mismatching.".to_string()));
            }
        }

        println!("res.results: {:?}", res.results());
        // store to cache for http gate to retrieve
        let data_to_cache = res.results().clone().unwrap_or("".to_string());
        _ = redis::set(
            &redis_addr,
            &TMP_CACHE_RESULTS.replace('#', reqid),
            data_to_cache.as_bytes(),
        );

        Ok(Some(pair_list.to_vec()))
    } else {
        let data_to_cache = res.results().clone().unwrap_or("".to_string());
        _ = redis::set(
            &redis_addr,
            &CACHE_STATUS_RESULTS.replace('#', &reqid),
            b"true",
        );
        _ = redis::set(
            &redis_addr,
            &CACHE_RESULTS.replace('#', &reqid),
            &data_to_cache.as_bytes(),
        );

        Ok(None)
    }
}

fn inner_stuffs_on_post_result(res: &EightFishResponse) -> Result<Vec<(String, String)>> {
    let pg_addr = std::env::var(DB_URL_ENV)?;
    let table_name = &res.info().model_name;
    //let id = &res.info.target;
    let action = &res.info().action;
    let id;
    let ins_hash;

    if res.pair_list().is_some() {
        // here, we just process single updated item returning
        let pair = &res.pair_list().clone().unwrap()[0];
        id = pair.0.clone();
        ins_hash = pair.1.clone();
    } else {
        bail!("No pair.".to_string());
    }

    if action == "new" {
        let sql_string = format!(
            "insert into {table_name}_idhash values ('{}', '{}')",
            id, ins_hash
        );
        let _execute_results = pg::execute(&pg_addr, &sql_string, &[]);
        // TODO: check the pg result
        println!(
            "in post stuff: new: _execute_results: {:?}",
            _execute_results
        );
    } else if action == "update" {
        let sql_string =
            format!("update {table_name}_idhash set hash='{ins_hash}' where id='{id}'");
        let _execute_results = pg::execute(&pg_addr, &sql_string, &[]);
        println!(
            "in post stuff: update: _execute_results: {:?}",
            _execute_results
        );
    } else if action == "delete" {
        let sql_string = format!("delete {table_name}_idhash where id='{id}'");
        let _execute_results = pg::execute(&pg_addr, &sql_string, &[]);
        // TODO: check the pg result
        println!(
            "in post stuff: delete: _execute_results: {:?}",
            _execute_results
        );
    } else {
    }

    let pair_list: Vec<(String, String)> = vec![(id, ins_hash)];

    Ok(pair_list)
}
