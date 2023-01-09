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

const REDIS_URL_ENV: &str = "REDIS_URL";
const DB_URL_ENV: &str = "DB_URL";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Article {
    id: String,
    title: String,
    content: String,
    authorname: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArticleHash {
    article: Article,
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
        self.article.id.to_string()
    }

    fn hash(&self) -> String {
        self.hash.to_string()
    }
}

pub struct Worker {
    
}

impl Worker {

    pub fn mount(app: EightFishApp) -> Result<()> {
    
    }

    pub fn work(message: Bytes) -> Result<()> {

    }

}

pub fn work(message: Bytes) -> Result<()> {

    // the message is the data retreived from the redis channel
    println!("{}", std::str::from_utf8(&message)?);

    // the message is a JSON stringified data
    // deserialize it
    let msg_obj: InputOutputObject = serde_json::from_slice(&message)?;

    match &msg_obj.action[..] {
        "query" => {
            // handle query request directly from the http_gate component
            // use postgres handle to query data from the postgres db
            let path = msg_obj.model.to_owned();
            
            // here, json_obj.data is a json obj with reqid and reqdata which is a string with url format
            let payload: Payload = serde_json::from_slice(&msg_obj.data)?;
            let _result = handle_query(&payload.reqid, &path, &payload.reqdata);
        }
        "post" => {
            // handle the result of the act call event
            // here, json_obj.data is a json obj with reqid and reqdata which is a string with form/url format
            let path = msg_obj.model.to_owned();
            let payload: Payload = serde_json::from_slice(&msg_obj.data)?;
            let _result = handle_event(&payload.reqid, &path, &payload.reqdata);
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

    Ok(())
}

fn handle_query(reqid: &str, path: &str, data: &Option<String>) -> Result<String> {
    let pg_addr = std::env::var(DB_URL_ENV)?;
    let redis_addr = std::env::var(REDIS_URL_ENV)?;
    
    // first part, we need to parse the params data into a structure
    let mut params: HashMap<String, String> = HashMap::new();
    
    if data.is_some() {
        let _parse = form_urlencoded::parse(&data.as_ref().unwrap().as_bytes());
        // Iterate this _parse, push values into params
        for pair in _parse {
            let key = pair.0.to_string();
            let val = pair.1.to_string();
            params.insert(key, val);
        }
    }

    match path {
        "/article" => {
            // ----- biz logic part -----
            // get the view of one article, the parameter is in 'data', in the format of url
            // encoded
            let article_id = params.get("id").unwrap();
            // construct a sql statement 
            let query_string = format!("select hash, id, title, content, author from article where id='{article_id}'");
            let rowset = pg::query(&pg_addr, &query_string, &vec![]).unwrap();

            // convert the raw vec[u8] to every rust struct filed, and convert the whole into a
            // rust struct vec, later we may find a gerneral type converter way
            let mut results: Vec<ArticleHash> = vec![];
            for row in rowset.rows {
                let id = as_owned_string(&row[1])?;
                let title = as_owned_string(&row[2])?;
                let content = as_owned_string(&row[3])?;
                let authorname = as_owned_string(&row[4])?;
                let hash = as_owned_string(&row[0])?;

                let article = Article {
                    id,
                    title,
                    content,
                    authorname,
                };

                // MUST check the article obj and the hash value equlity get from db
                let checked_hash = calc_hash(&article).unwrap();
                if checked_hash != hash {
                     return Err(anyhow!("Hash mismatching.".to_string()))
                }

                let article_hash = ArticleHash {
                    article,
                    hash,
                };

                println!("article_hash: {:#?}", article_hash);
                results.push(article_hash);

                tail_query_process(&redis_addr, reqid, "Article", &results);

            }
        }
        "/article_info" => {

        }
        &_ => {
            todo!()
        }   
    }

    Ok("ok".to_string())
}

fn handle_event(reqid: &str, path: &str, data: &Option<String>) -> Result<String> {
    let pg_addr = std::env::var(DB_URL_ENV)?;
    let redis_addr = std::env::var(REDIS_URL_ENV)?;
    
    let mut params: HashMap<String, String> = HashMap::new();
    if data.is_some() {
        // first part, we need to parse the params data into a structure
        let _parse = form_urlencoded::parse(&data.as_ref().unwrap().as_bytes());

        // Iterate this _parse, push values into params
        for pair in _parse {
            let key = pair.0.to_string();
            let val = pair.1.to_string();
            params.insert(key, val);
        }
    }

    match path {
        "/article/new" => {
            let title = params.get("title").unwrap();
            let content = params.get("content").unwrap();
            let authorname = params.get("authorname").unwrap();

            let id = Uuid::new_v4().simple().to_string(); // uuid

            // construct a struct
            let article = Article {
                id: id.clone(),
                title: title.clone(),
                content: content.clone(),
                authorname: authorname.clone(),
            };

            // should ensure the serialization way is determined.
            // and the field hash won't participate the serialization
            let hash = calc_hash(&article).unwrap();

            // construct a sql statement 
            let sql_string = format!("insert into article values ({}, {}, {}, {}, {})", &hash, article.id, article.title, article.content, article.authorname);
            let _execute_results = pg::execute(&pg_addr, &sql_string, &vec![]);

            tail_post_process(&redis_addr, reqid, "Article", &id, &hash);
        }
        "/article/delete" => {
            let id = params.get("id").unwrap();

            // construct a sql statement 
            let sql_string = format!("delete article where id='{id}'");
            let _execute_results = pg::execute(&pg_addr, &sql_string, &vec![]);

            tail_post_process(&redis_addr, reqid, "Article", &id, "");
        }
        &_ => {
            todo!()
        }   

    }

    Ok("ok".to_string())
}


fn as_owned_string(value: &pg::DbValue) -> anyhow::Result<String> {
    match value {
        pg::DbValue::Str(s) => Ok(s.to_owned()),
        _ => Err(anyhow!("Expected string from database but got {:?}", value)),
    }
}

fn calc_hash<T: Serialize>(obj: &T) -> Result<String> {
    // I think we can use json_digest to do the deterministic hash calculating
    // https://docs.rs/json-digest/0.0.16/json_digest/
    let json_val= serde_json::to_value(obj).unwrap();
    let digest = json_digest::digest_data(&json_val).unwrap();

    Ok(digest)
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
