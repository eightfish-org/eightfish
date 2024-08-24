use anyhow::{anyhow, Result};
use bytes::Bytes;
use eightfish::{
    App as EightFishApp, Handler, HandlerCRUD, Method, Request as EightFishRequest,
    Response as EightFishResponse,
};
use serde::Deserialize;
use serde_json::json;
use spin_sdk::{
    pg::{self, Decode},
    redis,
};
use std::collections::HashMap;

const REDIS_URL_ENV: &str = "REDIS_URL_ENV";
const DB_URL_ENV: &str = "DB_URL_ENV";
const TMP_CACHE_RESULTS: &str = "tmp:cache:#";
const CACHE_STATUS_RESULTS: &str = "cache:status:#";
const CACHE_RESULTS: &str = "cache:#";
const CHANNEL_GATE2VIN: &str = "gate2vin";

#[derive(Deserialize, Debug)]
pub struct InputOutputObject {
    proto: String,
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
                let pg_addr = std::env::var(DB_URL_ENV)?;
                let method = Method::Get;
                let proto_name = msg_obj.proto.to_owned();
                // path info put in the model field from the http_gate
                let path = msg_obj.model.to_owned();
                let payload: Payload = serde_json::from_slice(&msg_obj.data)?;
                let reqid = payload.reqid.to_owned();
                let reqdata = payload.reqdata;

                let mut ef_req = EightFishRequest::new(method, path, reqdata);
                println!("Worker::work: in query branch: ef_req");

                let ef_res = self.app.handle(&mut ef_req);
                match ef_res {
                    Ok(ef_res) => {
                        println!("Worker::work: in query branch: ef_res: {:?}", ef_res);
                        // we check the intermediate result  in the framework internal
                        let pair_list =
                            inner_stuffs_on_query_result(&redis_addr, &pg_addr, &reqid, &ef_res)
                                .unwrap();
                        println!("Worker::work: in query branch: pair_list: {:?}", pair_list);

                        if pair_list.is_some() {
                            let model_name = ef_res.info().model_name.to_owned();
                            // we can retrieve the model name from the path
                            // but that will force the developer use a strict unified url shcema in his product
                            // the names in query and post must MATCH
                            tail_query_process(
                                &redis_addr,
                                &reqid,
                                &proto_name,
                                &model_name,
                                &pair_list.unwrap(),
                            );
                        }
                    }
                    Err(err) => {
                        match err.downcast_ref::<&str>() {
                            Some(&"404") => {
                                // write not found msg to cache
                                _ = redis::set(
                                    &redis_addr,
                                    &CACHE_STATUS_RESULTS.replace('#', &reqid),
                                    b"404",
                                );
                                _ = redis::set(
                                    &redis_addr,
                                    &CACHE_RESULTS.replace('#', &reqid),
                                    b"Not Found",
                                );
                            }
                            Some(s) => {
                                // write not found msg to cache
                                _ = redis::set(
                                    &redis_addr,
                                    &CACHE_STATUS_RESULTS.replace('#', &reqid),
                                    b"500",
                                );
                                _ = redis::set(
                                    &redis_addr,
                                    &CACHE_RESULTS.replace('#', &reqid),
                                    &s.as_bytes(),
                                );
                            }
                            None => {
                                // write not found msg to cache
                                _ = redis::set(
                                    &redis_addr,
                                    &CACHE_STATUS_RESULTS.replace('#', &reqid),
                                    b"500",
                                );
                                _ = redis::set(
                                    &redis_addr,
                                    &CACHE_RESULTS.replace('#', &reqid),
                                    &format!("{}", err).as_bytes(),
                                );
                            }
                        }
                        return Err(anyhow!("query error"));
                    }
                }
            }
            "post" => {
                let redis_addr = std::env::var(REDIS_URL_ENV)?;
                let pg_addr = std::env::var(DB_URL_ENV)?;
                let method = Method::Post;
                let proto_name = msg_obj.proto.to_owned();
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
                // encode the vec<u8> as base58 string, and add random_str to req.ext
                let random_string = bs58::encode(&ext.randomvec).into_string();
                ef_req
                    .ext_mut()
                    .insert("random_str".to_string(), random_string);

                let ef_res = self.app.handle(&mut ef_req);
                match ef_res {
                    Ok(ef_res) => {
                        println!("Worker::work: in post branch: ef_res: {:?}", ef_res);
                        let pair_list =
                            inner_stuffs_on_post_result(&redis_addr, &pg_addr, &reqid, &ef_res)
                                .unwrap();
                        println!("Worker::work: in post branch: pair_list: {:?}", pair_list);

                        if !pair_list.is_empty() {
                            let model_name = ef_res.info().model_name.to_owned();
                            tail_post_process(
                                &redis_addr,
                                &reqid,
                                &proto_name,
                                &model_name,
                                &pair_list,
                            );
                        }
                    }
                    Err(err) => {
                        match err.downcast_ref::<&str>() {
                            Some(&"404") => {
                                // write not found msg to cache
                                _ = redis::set(
                                    &redis_addr,
                                    &CACHE_STATUS_RESULTS.replace('#', &reqid),
                                    b"404",
                                );
                                _ = redis::set(
                                    &redis_addr,
                                    &CACHE_RESULTS.replace('#', &reqid),
                                    b"Not Found",
                                );
                            }
                            Some(s) => {
                                // write not found msg to cache
                                _ = redis::set(
                                    &redis_addr,
                                    &CACHE_STATUS_RESULTS.replace('#', &reqid),
                                    b"500",
                                );
                                _ = redis::set(
                                    &redis_addr,
                                    &CACHE_RESULTS.replace('#', &reqid),
                                    &s.as_bytes(),
                                );
                            }
                            None => {
                                // write not found msg to cache
                                _ = redis::set(
                                    &redis_addr,
                                    &CACHE_STATUS_RESULTS.replace('#', &reqid),
                                    b"500",
                                );
                                _ = redis::set(
                                    &redis_addr,
                                    &CACHE_RESULTS.replace('#', &reqid),
                                    &format!("{}", err).as_bytes(),
                                );
                            }
                        }
                        return Err(anyhow!("post error"));
                    }
                }
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
                // let id = payload.reqdata.unwrap();

                // let result = json!({
                //     "result": "Ok",
                //     "id": id,
                // });

                // while getting the index updated callback, we put result http_gate wants into redis
                // cache
                let redis_addr = std::env::var(REDIS_URL_ENV)?;
                let cache_key = CACHE_STATUS_RESULTS.replace('#', &reqid);
                _ = redis::set(&redis_addr, &cache_key, b"200");

                // in previous post process, we have set the TMP_CACHE_RESULTS
                let tmpdata = redis::get(&redis_addr, &TMP_CACHE_RESULTS.replace('#', &reqid));
                if let Ok(tmpdata) = tmpdata {
                    // set to CACHE_RESULTS
                    let _ = redis::set(&redis_addr, &CACHE_RESULTS.replace('#', &reqid), &tmpdata);
                }
                // delete the tmp cache
                _ = redis::del(&redis_addr, &[&TMP_CACHE_RESULTS.replace('#', &reqid)]);
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
                        b"200",
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
                        b"400",
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
    proto_name: &str,
    model_name: &str,
    pair_list: &Vec<(String, String)>,
) {
    let payload = json!({
        "reqid": reqid,
        "reqdata": Some(pair_list),
    });

    println!("tail_query_process: payload: {:?}", payload);
    let json_to_send = json!({
        "proto": proto_name,
        "model": model_name,
        "action": "check_pair_list",
        "data": payload.to_string().as_bytes().to_vec(),
        "ext": Vec::<u8>::new(),
    });

    // send this to the redis channel to subxt to query rpc
    _ = redis::publish(
        redis_addr,
        CHANNEL_GATE2VIN,
        json_to_send.to_string().as_bytes(),
    );
}

fn tail_post_process(
    redis_addr: &str,
    reqid: &str,
    proto_name: &str,
    model_name: &str,
    pair_list: &Vec<(String, String)>,
) {
    let payload = json!({
        "reqid": reqid,
        "reqdata": Some(pair_list),
    });
    println!("tail_post_process: payload: {:?}", payload);

    let json_to_send = json!({
        "proto": proto_name,
        "model": model_name,
        "action": "update_index",
        "data": payload.to_string().as_bytes().to_vec(),
        "ext": Vec::<u8>::new(),
    });

    _ = redis::publish(
        redis_addr,
        CHANNEL_GATE2VIN,
        json_to_send.to_string().as_bytes(),
    );
}

fn inner_stuffs_on_query_result(
    redis_addr: &str,
    pg_addr: &str,
    reqid: &str,
    res: &EightFishResponse,
) -> Result<Option<Vec<(String, String)>>> {
    if res.pair_list().is_some() {
        let table_name = &res.info().model_name;
        let pair_list = res.pair_list().clone().unwrap();
        // get the id list from obj list
        let ids: Vec<String> = pair_list
            .iter()
            .map(|(id, _)| String::new() + "'" + &id[..] + "'")
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
        let data_to_cache = res.results().clone().unwrap_or("[]".to_string());
        _ = redis::set(
            &redis_addr,
            &CACHE_STATUS_RESULTS.replace('#', &reqid),
            b"200",
        );
        _ = redis::set(
            &redis_addr,
            &CACHE_RESULTS.replace('#', &reqid),
            &data_to_cache.as_bytes(),
        );

        Ok(None)
    }
}

fn inner_stuffs_on_post_result(
    redis_addr: &str,
    pg_addr: &str,
    reqid: &str,
    res: &EightFishResponse,
) -> Result<Vec<(String, String)>> {
    let table_name = &res.info().model_name;
    let action = &res.info().action;
    let id;
    let ins_hash;

    if res.pair_list().is_some() {
        // here, we just process single updated item returning
        let pair = &res.pair_list().clone().unwrap()[0];
        id = pair.0.clone();
        ins_hash = pair.1.clone();
    } else {
        let data_to_cache = "[]".to_string();
        _ = redis::set(
            &redis_addr,
            &CACHE_STATUS_RESULTS.replace('#', &reqid),
            b"200",
        );
        _ = redis::set(
            &redis_addr,
            &CACHE_RESULTS.replace('#', &reqid),
            &data_to_cache.as_bytes(),
        );

        return Ok(vec![]);
    }

    match action {
        HandlerCRUD::Create => {
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
        }
        HandlerCRUD::Update => {
            let sql_string =
                format!("update {table_name}_idhash set hash='{ins_hash}' where id='{id}'");
            let _execute_results = pg::execute(&pg_addr, &sql_string, &[]);
            println!(
                "in post stuff: update: _execute_results: {:?}",
                _execute_results
            );
        }
        HandlerCRUD::Delete => {
            let sql_string = format!("delete {table_name}_idhash where id='{id}'");
            let _execute_results = pg::execute(&pg_addr, &sql_string, &[]);
            // TODO: check the pg result
            println!(
                "in post stuff: delete: _execute_results: {:?}",
                _execute_results
            );
        }
        _ => unreachable!(),
    }

    // write response to tmp cache
    let data_to_cache = res.results().to_owned().unwrap_or("".to_string());
    _ = redis::set(
        &redis_addr,
        &TMP_CACHE_RESULTS.replace('#', reqid),
        data_to_cache.as_bytes(),
    );

    let pair_list: Vec<(String, String)> = vec![(id, ins_hash)];

    Ok(pair_list)
}
