use anyhow::{anyhow, Result};
use bytes::Bytes;
use eightfish_sdk::{
    App as EightFishApp, Handler, Method, Request as EightFishRequest,
    Response as EightFishResponse,
};
use http::{HeaderMap, StatusCode};
use serde::Deserialize;
use serde_json::json;
use spin_sdk::{
    redis::{self, RedisParameter, RedisResult},
    variables,
};
use std::collections::HashMap;

const REDIS_URL: &str = "REDIS_URL";
const DB_URL: &str = "DB_URL";
const CACHE_STATUS_RESULTS: &str = "cache:status:#";
const CACHE_HEADERS_RESULTS: &str = "cache:headers:#";
const CACHE_RESULTS: &str = "cache:#";
const CACHE_STACKCOUNT: &str = "cache:stackcount:#";
const CACHE_PAIRLIST: &str = "cache:pairlist:#";
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
    reqheaders: HashMap<String, String>,
}

#[derive(Deserialize, Debug)]
pub struct OnBlockHeightPayload {
    block_height: u64,
    block_hash: String,
}

#[derive(Deserialize, Debug)]
pub struct ExtPayload {
    block_height: u64,
    block_hash: String,
    time: u64,
    nonce: u64,
    randomvec: Vec<u8>,
}

pub struct Worker {
    app: EightFishApp,
    on_block_height: Option<Box<dyn Fn(u64, String) + Send + Sync + 'static>>,
}

impl Worker {
    pub fn mount(app: EightFishApp) -> Self {
        Worker {
            app,
            on_block_height: None,
        }
    }

    // Method to set the closure
    pub fn set_on_block_height<F>(&mut self, closure: F)
    where
        F: Fn(u64, String) + Send + Sync + 'static,
    {
        self.on_block_height = Some(Box::new(closure));
    }

    pub fn work(&self, message: Bytes) -> Result<()> {
        let msg_obj: InputOutputObject = serde_json::from_slice(&message)?;
        // println!("Worker::work: msg_obj: {:?}", msg_obj);

        match &msg_obj.action[..] {
            ACTION_NEW_BLOCK_HEIGHT => {
                // use msg as a timer, tick on every block height
                let payload: OnBlockHeightPayload = serde_json::from_slice(&msg_obj.data)?;

                // println!("Block height: {}", payload.block_height);
                // println!("Block hash: {}", payload.block_hash);
                if let Some(closure) = &self.on_block_height {
                    closure(payload.block_height, payload.block_hash);
                }
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
                let redis_addr = std::env::var(REDIS_URL)?;
                println!("redis_addr: {}", redis_addr);
                let redis_conn = redis::Connection::open(&redis_addr)
                    .expect("error when open redis connection.");

                let method = Method::Get;
                let proto_name = msg_obj.proto.to_owned();
                // path info put in the model field from the http_gate
                let path = msg_obj.model.to_owned();
                println!("get path: {}", path);

                let payload: Payload = serde_json::from_slice(&msg_obj.data)?;
                let reqid = payload.reqid.to_owned();
                println!("reqid: {}", reqid);
                let reqdata = payload.reqdata;
                println!("reqdata: {:?}", reqdata);
                let headers: HeaderMap = (&payload.reqheaders).try_into()?;
                println!("reqheaders: {:?}", headers);

                let mut ef_req = EightFishRequest::new(
                    method,
                    path,
                    reqid.clone(),
                    headers,
                    Some(proto_name.clone()),
                    reqdata,
                );
                // println!("Worker::work: in query branch: ef_req");

                let ef_res = self.app.handle(&mut ef_req);
                match ef_res {
                    Ok(ef_res) => {
                        match ef_res.status_code() {
                            StatusCode::OK => {
                                // process successful status case
                                // store intermedia data to cache
                                store_result_to_cache(&redis_conn, &reqid, &ef_res);

                                if let &Some(ref avec) = ef_res.pair_list() {
                                    if !avec.is_empty() {
                                        let model_name = ef_res.model_name().as_ref().unwrap();
                                        check_pair_list_from_vintage(
                                            &redis_conn,
                                            &reqid,
                                            &proto_name,
                                            model_name,
                                            avec,
                                        );
                                    }
                                }
                            }
                            _ => {
                                let headers = ef_res.headers().clone();
                                // process failed status case
                                let data_to_cache =
                                    ef_res.custom_result().to_owned().unwrap_or_default();
                                set_cache_result(&redis_conn, &reqid, headers, &data_to_cache);
                                let status_code: u16 = ef_res.status_code().into();
                                set_cache_status_code(
                                    &redis_conn,
                                    &reqid,
                                    &status_code.to_string(),
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
                let redis_addr = std::env::var(REDIS_URL)?;
                println!("redis_addr: {}", redis_addr);
                let redis_conn = redis::Connection::open(&redis_addr)
                    .expect("error when open redis connection.");

                let pg_addr = std::env::var(DB_URL)?;
                println!("pg_addr: {}", pg_addr);
                // let pg_conn =
                //     pg::Connection::open(&pg_addr).expect("error when open pg connection.");

                let method = match &msg_obj.action[..] {
                    "post" => Method::Post,
                    "put" => Method::Put,
                    "delete" => Method::Delete,
                    _ => return Err(anyhow::anyhow!("wrong http method.")),
                };
                println!("in action write: method: {:?}", method);
                let proto_name = msg_obj.proto.to_owned();
                println!("in action write: proto: {:?}", proto_name);
                let path = msg_obj.model.to_owned();
                println!("in action write: path: {:?}", path);
                let payload: Payload = serde_json::from_slice(&msg_obj.data)?;
                let reqid = payload.reqid.to_owned();
                println!("in action write: reqid: {:?}", reqid);
                let reqdata = payload.reqdata;
                println!("in action write: reqdata: {:?}", reqdata);
                let headers: HeaderMap = (&payload.reqheaders).try_into()?;
                println!("reqheaders: {:?}", headers);

                let ext: ExtPayload = serde_json::from_slice(&msg_obj.ext)?;

                let mut ef_req = EightFishRequest::new(
                    method,
                    path,
                    reqid.clone(),
                    headers,
                    Some(proto_name.clone()),
                    reqdata,
                );

                // insert block_height, block_hash, time, nonce and random_str to req.ext
                ef_req
                    .ext_mut()
                    .insert("block_height".to_string(), ext.block_height.to_string());
                // encode the hex digitals as base58 string
                // let block_hash_str = bs58::encode(&ext.block_hash).into_string();
                ef_req
                    .ext_mut()
                    .insert("block_hash".to_string(), ext.block_hash);
                ef_req
                    .ext_mut()
                    .insert("timestamp".to_string(), ext.time.to_string());
                ef_req
                    .ext_mut()
                    .insert("nonce".to_string(), ext.nonce.to_string());
                // encode the vec<u8> as base58 string
                let random_string = bs58::encode(&ext.randomvec).into_string();
                ef_req
                    .ext_mut()
                    .insert("random_str".to_string(), random_string);
                println!("in action write: req.ext: {:?}", ef_req.ext());

                let ef_res = self.app.handle(&mut ef_req);
                match ef_res {
                    Ok(ef_res) => {
                        match ef_res.status_code() {
                            StatusCode::OK => {
                                // process successful status case
                                // store intermedia data to cache
                                store_result_to_cache(&redis_conn, &reqid, &ef_res);

                                if let &Some(ref avec) = ef_res.pair_list() {
                                    if !avec.is_empty() {
                                        // set pair_list to a cache
                                        let key = CACHE_PAIRLIST.replace('#', &reqid);
                                        let val = serde_json::to_vec(avec)
                                            .expect("error when serialize pair list");
                                        _ = redis_conn.set(&key, &val);
                                    }
                                }
                            }
                            _ => {
                                let headers = ef_res.headers().clone();
                                // process failed status case
                                let data_to_cache =
                                    ef_res.custom_result().to_owned().unwrap_or_default();
                                set_cache_result(&redis_conn, &reqid, headers, &data_to_cache);
                                let status_code: u16 = ef_res.status_code().into();
                                set_cache_status_code(
                                    &redis_conn,
                                    &reqid,
                                    &status_code.to_string(),
                                );
                            }
                        }
                    }
                    Err(err) => {
                        return err_process(err, &redis_conn, &reqid);
                    }
                }
            }
            ACTION_UPDATE_INDEX => {
                // Callback: handle the result of the update_index call event
                let payload: Payload = serde_json::from_slice(&msg_obj.data)?;
                println!("callback: update_index: payload: {:?}", payload);

                let reqid = payload.reqid;

                let redis_addr = std::env::var(REDIS_URL)?;
                let redis_conn = redis::Connection::open(&redis_addr)
                    .expect("error when open redis connection.");

                // for any write action, we seem them the same, and
                // decrease the stack count on each event of them arrives
                let key0 = CACHE_STACKCOUNT.replace("#", &reqid);
                let param1 = RedisParameter::Binary(key0.as_bytes().to_vec());
                let avec: Vec<i64> = redis_conn
                    .execute("decr", &[param1])?
                    .into_iter()
                    .map(|res| match res {
                        RedisResult::Int64(result) => result,
                        _ => -1,
                    })
                    .collect();
                if !avec.is_empty() {
                    let result = avec[0];
                    // when the stack count reaches 0, it says all indexes have been
                    // updated to the vintage
                    if result == 0 {
                        // for write case, we also need to check pair list for result
                        let key1 = CACHE_PAIRLIST.replace('#', &reqid);
                        if let Some(avec) = redis_conn.get(&key1)? {
                            let pair_list: Vec<(String, String)> = serde_json::from_slice(&avec)?;
                            if !pair_list.is_empty() {
                                check_pair_list_from_vintage(
                                    &redis_conn,
                                    &reqid,
                                    &msg_obj.proto,
                                    &msg_obj.model,
                                    &pair_list,
                                );
                            }
                        }
                        // clear the pair list cache
                        // _ = redis_conn.del(&[key1]);
                        // clear the stack count key
                        _ = redis_conn.del(&[key0, key1]);
                    }
                } else {
                    // do nothing
                    _ = redis_conn.del(&[key0]);
                }

                // TODO: we need handle the error case when vintage throws erros
                // put it in the future version
            }
            ACTION_CHECK_PAIR_LIST => {
                let redis_addr = std::env::var(REDIS_URL)?;
                let redis_conn = redis::Connection::open(&redis_addr)
                    .expect("error when open redis connection.");

                // handle the result of the check_pair_list
                let payload: Payload = serde_json::from_slice(&msg_obj.data)?;
                let reqid = payload.reqid;
                let reqdata = payload.reqdata.unwrap_or_default();
                println!(
                    "on action check pair list: reqid reqdata: {:?} {:?}",
                    reqid, reqdata
                );

                if &reqdata == "true" {
                    let status_code = StatusCode::OK.as_u16();

                    // set cache status to make response data ready
                    set_cache_status_code(&redis_conn, &reqid, &status_code.to_string());
                } else {
                    let data = "Checking pair list failed, abort the response!";
                    set_cache_result(&redis_conn, &reqid, None, data);
                    let status_code = StatusCode::INTERNAL_SERVER_ERROR.as_u16();
                    set_cache_status_code(&redis_conn, &reqid, &status_code.to_string());
                }
            }
            &_ => {
                todo!()
            }
        }

        Ok(())
    }
}

fn store_result_to_cache(redis_conn: &redis::Connection, reqid: &str, res: &EightFishResponse) {
    let headers = res.headers().clone();
    if let &Some(ref avec) = res.pair_list() {
        if !avec.is_empty() {
            let data_to_cache = res.result().as_ref().unwrap();

            // here we do not set the status code, when check_pair_list returns, get it
            set_cache_result(redis_conn, reqid, headers, data_to_cache);
        } else {
            // if data array to return is an empty vector, return it immediately
            let data_to_cache = "[]";
            set_cache_result(redis_conn, reqid, headers, data_to_cache);
            set_cache_status_code(&redis_conn, &reqid, "200");
        }
    } else {
        // for the custom result, return it immediately
        if let &Some(ref astr) = res.custom_result() {
            // return customized string body
            let data_to_cache = astr;
            set_cache_result(redis_conn, reqid, headers, data_to_cache);
            set_cache_status_code(&redis_conn, &reqid, "200");
        } else {
            let data_to_cache = "none";
            set_cache_result(redis_conn, reqid, headers, data_to_cache);
            set_cache_status_code(&redis_conn, &reqid, "200");
        }
    }
}

fn set_cache_result(
    redis_conn: &redis::Connection,
    reqid: &str,
    headers: Option<HeaderMap>,
    data_to_cache: &str,
) {
    if headers.is_none() {
        _ = redis_conn.set(&CACHE_HEADERS_RESULTS.replace('#', &reqid), &vec![]);
    } else {
        let headers_map: HashMap<String, String> = headers
            .unwrap()
            .into_iter()
            .filter_map(|(name, value)| {
                name.map(|n| {
                    (
                        n.as_str().to_string(),
                        value.to_str().unwrap_or_default().to_string(),
                    )
                })
            })
            .collect();
        let headers_vec = serde_json::to_vec(&headers_map).unwrap_or_else(|_| Vec::new());

        _ = redis_conn.set(&CACHE_HEADERS_RESULTS.replace('#', &reqid), &headers_vec);
    }
    _ = redis_conn.set(
        &CACHE_RESULTS.replace('#', &reqid),
        &data_to_cache.as_bytes().to_vec(),
    );
}

fn set_cache_status_code(redis_conn: &redis::Connection, reqid: &str, status_code: &str) {
    _ = redis_conn.set(
        &CACHE_STATUS_RESULTS.replace('#', &reqid),
        &status_code.as_bytes().to_vec(),
    );
}

fn check_pair_list_from_vintage(
    redis_conn: &redis::Connection,
    reqid: &str,
    proto_name: &str,
    model_name: &str,
    pair_list: &Vec<(String, String)>,
) {
    let payload = json!({
        "reqid": reqid,
        "reqdata": Some(pair_list),
    });
    println!("check_pair_list_from_vintage: payload: {:?}", payload);

    let json_to_send = json!({
        "proto": proto_name,
        "model": model_name,
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

fn err_process(err: anyhow::Error, redis_conn: &redis::Connection, reqid: &str) -> Result<()> {
    println!("handler error: {:?}", err);
    match err.downcast_ref::<&str>() {
        Some(&"404") => {
            // write not found msg to cache
            _ = redis_conn.set(&CACHE_RESULTS.replace('#', &reqid), &b"Not Found".to_vec());
            _ = redis_conn.set(&CACHE_STATUS_RESULTS.replace('#', &reqid), &b"404".to_vec());
        }
        Some(s) => {
            // FIXME: logical error? we must choose a better status code scheme
            _ = redis_conn.set(&CACHE_RESULTS.replace('#', &reqid), &s.as_bytes().to_vec());
            _ = redis_conn.set(&CACHE_STATUS_RESULTS.replace('#', &reqid), &b"500".to_vec());
        }
        None => {
            // other errors
            _ = redis_conn.set(
                &CACHE_RESULTS.replace('#', &reqid),
                &format!("{}", err).as_bytes().to_vec(),
            );
            _ = redis_conn.set(&CACHE_STATUS_RESULTS.replace('#', &reqid), &b"500".to_vec());
        }
    }

    Err(anyhow!("handler error"))
}

#[allow(dead_code)]
pub fn update_index_on_write(
    redis_conn: &redis::Connection,
    reqid: &str,
    proto_name: &str,
    model_name: &str,
    pair_list: &Vec<(String, String)>,
) {
    let payload = json!({
        "reqid": reqid,
        "reqdata": Some(pair_list),
    });
    println!("update_index_on_write: payload: {:?}", payload);

    let json_to_send = json!({
        "proto": proto_name,
        "model": model_name,
        "action": "update_index",
        "data": payload.to_string().as_bytes().to_vec(),
        "ext": Vec::<u8>::new(),
    });

    _ = redis_conn.publish(
        CHANNEL_GATE2VIN,
        &json_to_send.to_string().as_bytes().to_vec(),
    );

    // increase write stack count
    _ = redis_conn.incr(&CACHE_STACKCOUNT.replace('#', reqid));
}

#[allow(dead_code)]
pub fn append_returning_star(sql: &str) -> String {
    let trimmed = sql.trim();
    if trimmed.to_uppercase().ends_with("RETURNING *") {
        trimmed.to_string()
    } else {
        format!("{} RETURNING *", trimmed)
    }
}

#[macro_export]
macro_rules! sql_create_one {
    ($req:expr, $instance:expr) => {{
        use spin_sdk::{pg, redis};

        let pg_addr = std::env::var(DB_URL).expect("ENV DB_URL not set.");
        let pg_conn = pg::Connection::open(&pg_addr).expect("error when open pg connection.");

        let (sql_statement, sql_params) = $instance.build_insert();
        let res = pg_conn.query(&sql_statement, &sql_params);
        match res {
            Ok(_) => {
                let instance_id = $instance.id();
                let instance_hash = $instance.calc_hash();
                let pair_list = vec![(instance_id, instance_hash)];

                let table_name = $instance.model_name();
                let reqid = $req.id();
                let proto_name = $req.proto().to_owned().unwrap_or_default();
                let redis_addr = std::env::var(REDIS_URL).expect("ENV REDIS_URLnot set.");
                let redis_conn = redis::Connection::open(&redis_addr)
                    .expect("error when open redis connection.");

                // update index to vintage
                spin_worker::update_index_on_write(
                    &redis_conn,
                    &reqid,
                    &proto_name,
                    &table_name,
                    &pair_list,
                );

                Ok($instance)
            }
            Err(e) => Err(e),
        }
    }};
}

#[macro_export]
macro_rules! sql_update_one {
    ($req:expr, $instance:expr) => {{
        use spin_sdk::{pg, redis};

        let pg_addr = std::env::var(DB_URL).expect("ENV DB_URLnot set.");
        let pg_conn = pg::Connection::open(&pg_addr).expect("error when open pg connection.");

        let (sql_statement, sql_params) = $instance.build_update();
        println!(
            "in sql_update_one!. sql_statement, sql_params: {:?} {:?}",
            sql_statement, sql_params
        );
        let res = pg_conn.query(&sql_statement, &sql_params);
        match res {
            Ok(_) => {
                let instance_id = $instance.id();
                let instance_hash = $instance.calc_hash();
                // recalculate the new instance's id hash pair
                let pair_list = vec![(instance_id, instance_hash)];

                let table_name = $instance.model_name();
                let reqid = $req.id();
                let proto_name = $req.proto().to_owned().unwrap();
                let redis_addr = std::env::var(REDIS_URL).expect("ENV REDIS_URL not set.");
                let redis_conn = redis::Connection::open(&redis_addr)
                    .expect("error when open redis connection.");

                // update index to vintage
                spin_worker::update_index_on_write(
                    &redis_conn,
                    &reqid,
                    &proto_name,
                    &table_name,
                    &pair_list,
                );

                Ok($instance)
            }
            Err(e) => Err(e),
        }
    }};
}

#[macro_export]
macro_rules! sql_update {
    ($req:expr, $model:ty, $sql_statement:expr, $sql_params:expr) => {{
        use spin_sdk::{pg, redis};

        let pg_addr = std::env::var(DB_URL).expect("ENV DB_URL not set.");
        let pg_conn = pg::Connection::open(&pg_addr).expect("error when open pg connection.");

        let sql_str = spin_worker::append_returning_star($sql_statement);
        println!("in sql_update, sql str: {}", sql_str);

        let res = pg_conn.query(&sql_str, $sql_params);
        match res {
            Ok(rowset) => {
                let mut instances = vec![];
                for row in rowset.rows {
                    let instance = <$model>::from_row(row);
                    instances.push(instance);
                }

                if !instances.is_empty() {
                    let mut pair_list = vec![];

                    // collect affected rows' id and hash
                    for instance in &instances {
                        let instance_id = instance.id();
                        let instance_hash = instance.calc_hash();
                        pair_list.push((instance_id, instance_hash));
                    }

                    let table_name = <$model>::model_name();
                    let reqid = $req.id();
                    let proto_name = $req.proto().to_owned().unwrap();
                    let redis_addr = std::env::var(REDIS_URL).expect("ENV REDIS_URL not set.");
                    let redis_conn = redis::Connection::open(&redis_addr)
                        .expect("error when open redis connection.");

                    // update index to vintage
                    spin_worker::update_index_on_write(
                        &redis_conn,
                        &reqid,
                        &proto_name,
                        &table_name,
                        &pair_list,
                    );

                    Ok(instances)
                } else {
                    Ok(vec![])
                }
            }
            Err(e) => Err(e),
        }
    }};
}

#[macro_export]
macro_rules! sql_delete_one {
    ($req:expr, $instance:expr) => {{
        use spin_sdk::{pg, redis};

        let pg_addr = std::env::var(DB_URL).expect("ENV DB_URL not set.");
        let pg_conn = pg::Connection::open(&pg_addr).expect("error when open pg connection.");

        let (sql_statement, sql_params) = $instance.build_delete();
        let res = pg_conn.query(&sql_statement, &sql_params);
        match res {
            Ok(_) => {
                // recalculate the new instance's id hash pair
                let pair_list = vec![(instance_id, "".to_string())];

                let table_name = $instance.model_name();
                let reqid = $req.id();
                let proto_name = $req.proto().to_owned().unwrap();

                let redis_addr = std::env::var(REDIS_URL).expect("ENV REDIS_URL not set.");
                let redis_conn = redis::Connection::open(&redis_addr)
                    .expect("error when open redis connection.");

                // update index to vintage
                spin_worker::update_index_on_write(
                    &redis_conn,
                    &reqid,
                    &proto_name,
                    &table_name,
                    &pair_list,
                );

                Ok($instance)
            }
            Err(e) => Err(e),
        }
    }};
}

#[macro_export]
macro_rules! sql_delete {
    ($req:expr, $model:ty, $sql_statement:expr, $sql_params:expr) => {{
        use spin_sdk::{pg, redis};

        let pg_addr = std::env::var(DB_URL).expect("ENV DB_URL not set.");
        let pg_conn = pg::Connection::open(&pg_addr).expect("error when open pg connection.");

        let sql_str = spin_worker::append_returning_star($sql_statement);
        println!("in sql_delete, sql str: {}", sql_str);

        let res = pg_conn.query(&sql_str, $sql_params);
        match res {
            Ok(rowset) => {
                let mut instances = vec![];
                for row in rowset.rows {
                    let instance = <$model>::from_row(row);
                    instances.push(instance);
                }

                if !instances.is_empty() {
                    let table_name = <$model>::model_name();

                    let mut ids = vec![];
                    for instance in &instances {
                        let instance_id = instance.id();
                        ids.push(instance_id);
                    }
                    let pair_list = ids.into_iter().map(|id| (id, "".to_string())).collect();

                    let reqid = $req.id();
                    let proto_name = $req.proto().to_owned().unwrap();

                    let redis_addr = std::env::var(REDIS_URL).expect("ENV REDIS_URL not set.");
                    // println!("redis_addr: {}", redis_addr);
                    let redis_conn = redis::Connection::open(&redis_addr)
                        .expect("error when open redis connection.");

                    // update index to vintage
                    spin_worker::update_index_on_write(
                        &redis_conn,
                        &reqid,
                        &proto_name,
                        &table_name,
                        &pair_list,
                    );

                    Ok(instances)
                } else {
                    Ok(vec![])
                }
            }
            Err(e) => Err(e),
        }
    }};
}

#[macro_export]
macro_rules! sql_query_one {
    ($model: ty, $id: expr) => {{
        use spin_sdk::pg;
        use std::collections::HashMap;

        let pg_addr = std::env::var(DB_URL).expect("ENV DB_URL not set.");
        let pg_conn = pg::Connection::open(&pg_addr).expect("error when open pg connection.");

        let (sql, sql_params) = <$model>::build_get_by_id($id);
        let rowset = pg_conn.query(&sql, &sql_params)?;

        if let Some(row) = rowset.rows.into_iter().next() {
            let instance = <$model>::from_row(row);

            Some(instance)
        } else {
            None
        }
    }};
}

#[macro_export]
macro_rules! sql_query {
    ($model:ty, $sql_statement:expr, $sql_params:expr) => {{
        use spin_sdk::pg;
        use std::collections::HashMap;

        let pg_addr = std::env::var(DB_URL).expect("ENV DB_URL not set.");
        let pg_conn = pg::Connection::open(&pg_addr).expect("error when open pg connection.");

        let rowset = pg_conn.query($sql_statement, $sql_params)?;

        let mut instances = vec![];
        for row in rowset.rows.into_iter() {
            let instance = <$model>::from_row(row);
            instances.push(instance);
        }

        instances
    }};
}
