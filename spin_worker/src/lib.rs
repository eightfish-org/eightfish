use anyhow::{anyhow, Result};
use bytes::Bytes;
use eightfish_sdk::{
    App as EightFishApp, Handler, HandlerCRUD, Method, Request as EightFishRequest,
    Response as EightFishResponse,
};
use serde::Deserialize;
use serde_json::json;
use spin_sdk::{
    pg::{self, Decode},
    redis, variables,
};
use std::collections::HashMap;

const REDIS_URL_ENV: &str = "REDIS_URL_ENV";
const DB_URL_ENV: &str = "DB_URL_ENV";
const TMP_CACHE_RESULTS: &str = "tmp:cache:#";
const CACHE_STATUS_RESULTS: &str = "cache:status:#";
const CACHE_RESULTS: &str = "cache:#";
const CHANNEL_GATE2VIN: &str = "gate2vin";
const ACTION_NEW_BLOCK_HEIGHT: &str = "block_height";
const ACTION_UPLOAD_WASM: &str = "upload_wasm";
const ACTION_UPGRADE_WASM: &str = "upgrade_wasm";
const ACTION_GET: &str = "get";
const ACTION_POST: &str = "post";
const ACTION_PUT: &str = "put";
const ACTION_DELETE: &str = "delete";
const ACTION_UPDATE_INDEX: &str = "update_index";
const ACTION_CHECK_PAIR_LIST: &str = "check_pair_list";

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
        // println!("Worker::work: msg_obj: {:?}", msg_obj);

        match &msg_obj.action[..] {
            ACTION_NEW_BLOCK_HEIGHT => {
                // use msg as a timer, tick on every block height
                let body: [u8; 8] = msg_obj.data.try_into().unwrap_or([0; 8]);
                // convert to u64
                let _block_height = u64::from_be_bytes(body);

                // do something
                // println!("Block height: {block_height}");
            }
            ACTION_UPLOAD_WASM => {
                // do nothing, wasm_worker itself doesn't care about new wasm file uploaded
            }
            ACTION_UPGRADE_WASM => {
                // get proto_id from the spin variables
                let current_proto_id = variables::get("proto_id")?;
                let current_wasm_hash = variables::get("wasm_hash")?;
                println!(
                    "current proto_id: {current_proto_id}, current wasm_hash: {current_wasm_hash}"
                );
                // let proto_id = msg_obj.proto;
                // // if the upgrade msg is for me, do upgrade
                // if proto_id == current_proto_id {
                //     let wasm_hash = hex::encode(msg_obj.data);
                //     if wasm_hash != current_wasm_hash {
                //         println!("== ready to exit. ==");
                //         // exit from the wasm process
                //         std::process::exit(0);
                //     }
                // }
            }
            ACTION_GET => {
                let redis_addr = std::env::var(REDIS_URL_ENV)?;
                let redis_conn = redis::Connection::open(&redis_addr)
                    .expect("error when open redis connection.");

                let method = Method::Get;
                let proto_name = msg_obj.proto.to_owned();
                // path info put in the model field from the http_gate
                let path = msg_obj.model.to_owned();
                let payload: Payload = serde_json::from_slice(&msg_obj.data)?;
                let reqid = payload.reqid.to_owned();
                let reqdata = payload.reqdata;

                let mut ef_req = EightFishRequest::new(method, path, reqid, proto_name, reqdata);
                println!("Worker::work: in query branch: ef_req");

                let ef_res = self.app.handle(&mut ef_req);
                match ef_res {
                    Ok(ef_res) => {
                        println!("Worker::work: in query branch: ef_res: {:?}", ef_res);
                        // we check the intermediate result  in the framework internal
                        store_query_intermedia_result_to_cache(&redis_conn, &reqid, &ef_res);

                        if &Some(ref avec) = res.result() {
                            if !avec.is_empty() {
                                check_pair_list_from_vintage(
                                    &redis_conn,
                                    &reqid,
                                    &proto_name,
                                    ef_res,
                                );
                            }
                        }
                    }
                    Err(err) => {
                        return err_process(err, &redis_conn, &reqid);
                    }
                }
            }
            ACTION_POST | ACTION_PUT | ACTION_DELETE => {
                let redis_addr = std::env::var(REDIS_URL_ENV)?;
                println!("redis_addr: {}", redis_addr);
                let redis_conn = redis::Connection::open(&redis_addr)
                    .expect("error when open redis connection.");

                let pg_addr = std::env::var(DB_URL_ENV)?;
                println!("pg_addr: {}", pg_addr);
                let pg_conn =
                    pg::Connection::open(&pg_addr).expect("error when open pg connection.");

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
                        // store intermedia data to cache
                        store_result_to_cache(&redis_conn, &reqid, &ef_res);
                    }
                    Err(err) => {
                        return err_process(err, &redis_conn, &reqid);
                    }
                }
            }
            ACTION_UPDATE_INDEX => {
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
                let redis_conn = redis::Connection::open(&redis_addr)
                    .expect("error when open redis connection.");

                // in previous post process, we have set the TMP_CACHE_RESULTS
                let tmpdata = redis_conn.get(&TMP_CACHE_RESULTS.replace('#', &reqid));
                println!("callback: update_index: tmpdata: {:?}", tmpdata);
                if let Ok(Some(tmpdata)) = tmpdata {
                    // set to CACHE_RESULTS
                    let _ = redis_conn.set(&CACHE_RESULTS.replace('#', &reqid), &tmpdata);
                    // put status results behind to avoid the atomic result retrieving problem
                    let cache_key = CACHE_STATUS_RESULTS.replace('#', &reqid);
                    _ = redis_conn.set(&cache_key, &b"200".to_vec());
                }
                // delete the tmp cache
                _ = redis_conn.del(&[TMP_CACHE_RESULTS.replace('#', &reqid)]);
            }
            ACTION_CHECK_PAIR_LIST => {
                let redis_addr = std::env::var(REDIS_URL_ENV)?;
                let redis_conn = redis::Connection::open(&redis_addr)
                    .expect("error when open redis connection.");

                // handle the result of the check_pair_list
                let payload: Payload = serde_json::from_slice(&msg_obj.data)?;
                let reqid = payload.reqid.clone();
                let reqdata = payload.reqdata.unwrap();

                if &reqdata == "true" {
                    // check pass, get content from the tmp cache and write this content to a cache
                    let tmpdata = redis_conn.get(&TMP_CACHE_RESULTS.replace('#', &reqid));
                    if let Ok(Some(tmpdata)) = tmpdata {
                        let _ = redis_conn.set(&CACHE_RESULTS.replace('#', &reqid), &tmpdata);
                        // put status results behind to avoid the atomic result retrieving problem
                        _ = redis_conn
                            .set(&CACHE_STATUS_RESULTS.replace('#', &reqid), &b"200".to_vec());
                    }
                    // delete the tmp cache
                    _ = redis_conn.del(&[TMP_CACHE_RESULTS.replace('#', &reqid)]);
                } else {
                    let data = "check of pair list wrong!";
                    _ = redis_conn.set(
                        &CACHE_RESULTS.replace('#', &reqid),
                        &data.as_bytes().to_vec(),
                    );
                    // put status results behind to avoid the atomic result retrieving problem
                    _ = redis_conn
                        .set(&CACHE_STATUS_RESULTS.replace('#', &reqid), &b"400".to_vec());
                    // clear left tmp cache key
                    _ = redis_conn.del(&[TMP_CACHE_RESULTS.replace('#', &reqid)]);
                }
            }
            &_ => {
                todo!()
            }
        }

        Ok(())
    }
}

fn store_query_intermedia_result_to_cache(
    redis_conn: &redis::Connection,
    reqid: &str,
    res: &EightFishResponse,
) {
    if &Some(ref avec) = res.result() {
        if !avec.is_empty() {
            let data_to_cache = serde_json::to_string(avec)
                .expect("error when do serde_json serialization.")

            // store to tmp cache, when check_pair_list returns, get it
            _ = redis_conn.set(
                &TMP_CACHE_RESULTS.replace('#', reqid),
                &data_to_cache.as_bytes().to_vec(),
            );
        }
        else {
            let data_to_cache = "[]".to_string();
            _ = redis_conn.set(
                &CACHE_RESULTS.replace('#', &reqid),
                &data_to_cache.as_bytes().to_vec(),
            );
            _ = redis_conn.set(&CACHE_STATUS_RESULTS.replace('#', &reqid), &b"200".to_vec());
        }
    } else {
        let data_to_cache = "none".to_string();
        _ = redis_conn.set(
            &CACHE_RESULTS.replace('#', &reqid),
            &data_to_cache.as_bytes().to_vec(),
        );
        _ = redis_conn.set(&CACHE_STATUS_RESULTS.replace('#', &reqid), &b"200".to_vec());
    }
}

fn store_result_to_cache(
    redis_conn: &redis::Connection,
    reqid: &str,
    res: &EightFishResponse,
) {
    if &Some(ref avec) = res.result() {
        let data_to_cache = serde_json::to_string(avec)
            .expect("error when do serde_json serialization.")

        // // store to cache for http gate to retrieve
        // _ = redis_conn.set(
        //     &TMP_CACHE_RESULTS.replace('#', reqid),
        //     &data_to_cache.as_bytes().to_vec(),
        // );

        _ = redis_conn.set(
            &CACHE_RESULTS.replace('#', &reqid),
            &data_to_cache.as_bytes().to_vec(),
        );
        _ = redis_conn.set(&CACHE_STATUS_RESULTS.replace('#', &reqid), &b"200".to_vec());
        
    } else {
        let data_to_cache = "none".to_string();
        _ = redis_conn.set(
            &CACHE_RESULTS.replace('#', &reqid),
            &data_to_cache.as_bytes().to_vec(),
        );
        _ = redis_conn.set(&CACHE_STATUS_RESULTS.replace('#', &reqid), &b"200".to_vec());
    }
}

fn update_index_on_write(
    redis_conn: &redis::Connection,
    reqid: &str,
    proto_name: &str,
    triple_list: &Vec<(String, String, String)>,
) {
    let payload = json!({
        "reqid": reqid,
        "reqdata": Some(triple_list),
    });
    println!("update_index_on_write: payload: {:?}", payload);

    let json_to_send = json!({
        "proto": proto_name,
        "model": "",                // model name info has been put into payload reqdata
        "action": "update_index",
        "data": payload.to_string().as_bytes().to_vec(),
        "ext": Vec::<u8>::new(),
    });

    _ = redis_conn.publish(
        CHANNEL_GATE2VIN,
        &json_to_send.to_string().as_bytes().to_vec(),
    );
}

fn check_pair_list_from_vintage<T: EightFishModel + Serialize>(
    redis_conn: &redis::Connection,
    reqid: &str,
    proto_name: &str,
    ef_res: &EightFishResponse<T>,
) {
    let &Some(ref data) = ef_res.result() {
        if !data.is_empty() {
            let pair_list = data.map(|&elem| (elem.model_name(), elem.id(), elem.calc_hash())).collect();

            let payload = json!({
                "reqid": reqid,
                "reqdata": Some(pair_list),
            });
            println!("check_pair_list_from_vintage: payload: {:?}", payload);
            
            let json_to_send = json!({
                "proto": proto_name,
                "model": "",
                "action": "check_pair_list",
                "data": payload.to_string().as_bytes().to_vec(),
                "ext": Vec::<u8>::new(),
            });
        
            // send this to the redis channel to subxt to query rpc
            _ = redis_conn.publish(
                CHANNEL_GATE2VIN,
                &json_to_send.to_string().as_bytes().to_vec(),
            );
        }
    }
}

fn err_process(err: anyhow::Error, redis_conn: &redis::Connection, reqid: &str) -> Result<()> { 
    match err.downcast_ref::<&str>() {
        Some(&"404") => {
            // write not found msg to cache
            _ = redis_conn.set(&CACHE_RESULTS.replace('#', &reqid), &b"Not Found".to_vec());
            _ = redis_conn.set(&CACHE_STATUS_RESULTS.replace('#', &reqid), &b"404".to_vec());
        }
        Some(s) => {
            // write not found msg to cache
            _ = redis_conn.set(&CACHE_RESULTS.replace('#', &reqid), &s.as_bytes().to_vec());
            _ = redis_conn.set(&CACHE_STATUS_RESULTS.replace('#', &reqid), &b"500".to_vec());
        }
        None => {
            // write not found msg to cache
            _ = redis_conn.set(
                &CACHE_RESULTS.replace('#', &reqid),
                &format!("{}", err).as_bytes().to_vec(),
            );
            _ = redis_conn.set(&CACHE_STATUS_RESULTS.replace('#', &reqid), &b"500".to_vec());
        }
    }

    Err(anyhow!("handler error"))
}

#[macro_export]
macro_rules! sql_create_one {
    ($instance:expr) => {
        let pg_addr = std::env::var(DB_URL_ENV).expect("ENV DB_URL_ENV not set.");
        // println!("pg_addr: {}", pg_addr);
        let pg_conn =
            pg::Connection::open(&pg_addr).expect("error when open pg connection.");

        let (sql_statement, sql_params) = $instance.build_insert();
        let res = pg_conn.execute(&sql_statement, &sql_params)?;
        match res {
            Ok(_) => {
                // update associated idhash table
                let table_name = $instance.model_name();
                let instance_id = $instance.id();
                let instance_hash = $instance.calc_hash();
                let sql_statement = format!(
                    "insert into {table_name}_idhash values ('{}', '{}')",
                    instance_id, instance_hash
                );
                let res = pg_conn.execute(&sql_statement, &[]);
                match res {
                    Ok(_) => {
                        // recalculate the new instance's id hash pair
                        let hash_triple = (table_name, instance_id, instance_hash);
                        
                        // assume there is always an instance named req in every handler context
                        let reqid = req.id();
                        let proto_name = req.proto();
                        let redis_addr = std::env::var(REDIS_URL_ENV).expect("ENV REDIS_URL_ENV not set.");
                        // println!("redis_addr: {}", redis_addr);
                        let redis_conn = redis::Connection::open(&redis_addr)
                            .expect("error when open redis connection.");

                        // update index to vintage
                        update_index_on_write(&redis_conn, &reqid, &proto_name, &vec![hash_triple]);

                    }
                    Err(_) => {}
                }
            }
            Err(_) => {}
        }
    };
}

#[macro_export]
macro_rules! sql_update_one {
    ($instance:expr) => {
        let pg_addr = std::env::var(DB_URL_ENV).expect("ENV DB_URL_ENV not set.");
        // println!("pg_addr: {}", pg_addr);
        let pg_conn =
            pg::Connection::open(&pg_addr).expect("error when open pg connection.");

        let (sql_statement, sql_params) = $instance.build_update_one();
        let res = pg_conn.execute(&sql_statement, &sql_params)?;
        match res {
            Ok(_) => {
                // update associated idhash table
                let table_name = $instance.model_name();
                let instance_id = $instance.id();
                let instance_hash = $instance.calc_hash();
                let sql_statement = format!("update {table_name}_idhash set hash='{instance_hash}' where id='{instance_id}'");
                let res = pg_conn.execute(&sql_statement, &[]);
                match res {
                    Ok(_) => {
                        // recalculate the new instance's id hash pair
                        let hash_triple = (table_name, instance_id, instance_hash);
                        
                        // assume there is always an instance named req in every handler context
                        let reqid = req.id();
                        let proto_name = req.proto();
                        let redis_addr = std::env::var(REDIS_URL_ENV).expect("ENV REDIS_URL_ENV not set.");
                        // println!("redis_addr: {}", redis_addr);
                        let redis_conn = redis::Connection::open(&redis_addr)
                            .expect("error when open redis connection.");

                        // update index to vintage
                        update_index_on_write(&redis_conn, &reqid, &proto_name, &vec![hash_triple]);

                    }
                    Err(_) => {}
                }
            }
            Err(_) => {}
        }
    };
}

#[macro_export]
macro_rules! sql_update {
    ($model: ty, $sql_statement: expr, $sql_params: expr) => {
        let pg_addr = std::env::var(DB_URL_ENV).expect("ENV DB_URL_ENV not set.");
        // println!("pg_addr: {}", pg_addr);
        let pg_conn =
            pg::Connection::open(&pg_addr).expect("error when open pg connection.");

        let res = pg_conn.execute(&sql_statement, &sql_params)?;
        match res {
            Ok(rows) => {
                let table_name = $model::model_name();

                // MAKE SURE we get affected rows from the db update
                let instances = $model::from_rows(rows);
                // update associated idhash table
                
                let mut values = vec![];
                let mut values_str = vec![];
                
                // collect affected rows' id and hash
                for instance in instances {
                    let instance_id = instance.id();
                    let instance_hash = instance.calc_hash();
                    values_str.push(format!("('{}', '{}')", instance_id, instance_hash));
                    values.push((instance_id, instance_hash));
                }
                // construct a sql to update all associated rows in idhash table
                let values_str = values_str.concat(", ");
                let sql_statement = format!(#"
                update {}_idhash set hash=v.hash FROM (VALUES
                    {}
                ) AS v(id, hash)
                where id=v.id
                "#, table_name, values_str);
                let res = pg_conn.execute(&sql_statement, &[]);
                match res {
                    Ok(_) => {
                        let triples = values.iter().map(|(&id, &hash)| {
                            (table_name.clone(), id.clone(), hash.clone())
                        }).collect();
                        
                        // assume there is always an instance named req in every handler context
                        let reqid = req.id();
                        let proto_name = req.proto();
                        let redis_addr = std::env::var(REDIS_URL_ENV).expect("ENV REDIS_URL_ENV not set.");
                        // println!("redis_addr: {}", redis_addr);
                        let redis_conn = redis::Connection::open(&redis_addr)
                            .expect("error when open redis connection.");

                        // update index to vintage
                        update_index_on_write(&redis_conn, &reqid, &proto_name, &triples);

                    }
                    Err(_) => {}
                }
            }
            Err(_) => {}
        }
    };
}

#[macro_export]
macro_rules! sql_delete_one {
    ($instance:expr) => {
        let pg_addr = std::env::var(DB_URL_ENV).expect("ENV DB_URL_ENV not set.");
        // println!("pg_addr: {}", pg_addr);
        let pg_conn =
            pg::Connection::open(&pg_addr).expect("error when open pg connection.");

        let (sql_statement, sql_params) = $instance.build_delete_one();
        let res = pg_conn.execute(&sql_statement, &sql_params)?;
        match res {
            Ok(_) => {
                // update associated idhash table
                let table_name = $instance.model_name();
                let instance_id = $instance.id();
                // let instance_hash = $instance.calc_hash();
                let sql_statement = format!("delete {}_idhash where id='{}'", table_name, instance_id);
                let res = pg_conn.execute(&sql_statement, &[]);
                match res {
                    Ok(_) => {
                        // recalculate the new instance's id hash pair
                        let hash_triple = (table_name, instance_id, "".to_string());
                        
                        // assume there is always an instance named req in every handler context
                        let reqid = req.id();
                        let proto_name = req.proto();

                        let redis_addr = std::env::var(REDIS_URL_ENV).expect("ENV REDIS_URL_ENV not set.");
                        // println!("redis_addr: {}", redis_addr);
                        let redis_conn = redis::Connection::open(&redis_addr)
                            .expect("error when open redis connection.");

                        // update index to vintage
                        update_index_on_write(&redis_conn, &reqid, &proto_name, &vec![hash_triple]);

                    }
                    Err(_) => {}
                }
            }
            Err(_) => {}
        }
    };
}

#[macro_export]
macro_rules! sql_delete {
    ($model: ty, $sql_statement: expr, $sql_params: expr) => {
        let pg_addr = std::env::var(DB_URL_ENV).expect("ENV DB_URL_ENV not set.");
        // println!("pg_addr: {}", pg_addr);
        let pg_conn =
            pg::Connection::open(&pg_addr).expect("error when open pg connection.");

        let res = pg_conn.execute(&sql_statement, &sql_params)?;
        match res {
            Ok(rows) => {
                let table_name = $model::model_name();

                // MAKE SURE we get affected rows from the db update
                let instances = $model::from_rows(rows);
                // update associated idhash table
                
                let mut values = vec![];
                let mut values_str = vec![];
                
                // collect affected rows' id and hash
                for instance in instances {
                    let instance_id = instance.id();
                    // let instance_hash = instance.calc_hash();
                    values_str.push(format!("'{}'", instance_id));
                    values.push(instance_id);
                }
                // construct a sql to update all associated rows in idhash table
                let values_str = values_str.concat(", ");
                let sql_statement = format!(#"delete {}_idhash where id in ({})"#, table_name, values_str);
                let res = pg_conn.execute(&sql_statement, &[]);
                match res {
                    Ok(_) => {
                        let triples = values.iter().map(|(&id, &hash)| {
                            (table_name.clone(), id.clone(), "".to_string())
                        }).collect();
                        
                        // assume there is always an instance named req in every handler context
                        let reqid = req.id();
                        let proto_name = req.proto();

                        let redis_addr = std::env::var(REDIS_URL_ENV).expect("ENV REDIS_URL_ENV not set.");
                        // println!("redis_addr: {}", redis_addr);
                        let redis_conn = redis::Connection::open(&redis_addr)
                            .expect("error when open redis connection.");

                        // update index to vintage
                        update_index_on_write(&redis_conn, &reqid, &proto_name, &triples);

                    }
                    Err(_) => {}
                }
            }
            Err(_) => {}
        }
    };
}

#[macro_export]
macro_rules! sql_query_one {
    ($model: ty, $id: expr) => {
        let pg_addr = std::env::var(DB_URL_ENV).expect("ENV DB_URL_ENV not set.");
        // println!("pg_addr: {}", pg_addr);
        let pg_conn =
            pg::Connection::open(&pg_addr).expect("error when open pg connection.");

        let (sql, sql_params) = $model::build_sql_query_by_id($id);
        let rowset = pg_conn.query(&sql, &sql_params)?;

        if let Some(row) = rowset.rows.into_iter().next() {
            let instance = $model::from_row(row);

            // step: we need to query from idhash table to compare
            let ids_string = format!("'{}'", $id);
            let query_string =
                format!("select id, hash from {table_name}_idhash where id in ({ids_string})");
            println!("query_string: {:?}", query_string);
            let rowset = pg_conn.query(&query_string, &[]).unwrap();

            let mut idhash_map: HashMap<String, String> = HashMap::new();
            for row in rowset.rows {
                let id = String::decode(&row[0])?;
                let hash = String::decode(&row[1])?;

                idhash_map.insert(id, hash);
            }

            let instances = vec![instance];

            // iterate on the input results to check
            for instance in instances {
                let id = instance.id();
                let hash = instance.calc_hash();
                let hash_from_map = idhash_map.get(&id[..]).expect("idhash_map get error");

                if hash != hash_from_map {
                    println!("compare hash, hash_from_map: {}, {}", hash, hash_from_map);
                    return Err(anyhow!("Hash mismatching.".to_string()));
                }
            }

            Some(instance)

        } else {
            // error
            // bail!("no this item".to_string());
            None
        };

    };
}

#[macro_export]
macro_rules! sql_query {
    ($model: ty, $sql_statement: expr, $sql_params: expr) => {
        let pg_addr = std::env::var(DB_URL_ENV).expect("ENV DB_URL_ENV not set.");
        // println!("pg_addr: {}", pg_addr);
        let pg_conn =
            pg::Connection::open(&pg_addr).expect("error when open pg connection.");

        let rowset = pg_conn.query(&sql_statement, &sql_params)?;

        let instances = vec![];
        for row in rowset.rows.into_iter() {
            let instance = $model::from_row(row);
            instances.push(instance);
        }

        if !instances.is_empty() {
            // get ids
            let ids = instances.map(|&instance| instance.id().to_owned()).collect();
            let mut ids_str = vec![];
                
            // collect affected rows' id and hash
            for id in ids {
                ids_str.push(format!("'{}'", id));
            }
            // step: we need to query from idhash table to compare
            let ids_string = ids_str.concat(", ");
            let query_string =
                format!("select id, hash from {table_name}_idhash where id in ({ids_string})");
            println!("query_string: {:?}", query_string);
            let rowset = pg_conn.query(&query_string, &[]).unwrap();

            let mut idhash_map: HashMap<String, String> = HashMap::new();
            for row in rowset.rows {
                let id = String::decode(&row[0])?;
                let hash = String::decode(&row[1])?;

                idhash_map.insert(id, hash);
            }

            // iterate on the input results to check
            for instance in instances {
                let id = instance.id();
                let hash = instance.calc_hash();
                let hash_from_map = idhash_map.get(&id[..]).expect("idhash_map get error");

                if hash != hash_from_map {
                    println!("compare hash, hash_from_map: {}, {}", hash, hash_from_map);
                    return Err(anyhow!("Hash mismatching.".to_string()));
                }
            }

            instances

        } else {
            // error
            // bail!("no this item".to_string());
            instances
        };    
    };
}
