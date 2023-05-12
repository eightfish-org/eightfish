#![allow(unused_assignments)]

use anyhow::{anyhow, Result};
use bytes::Bytes;
use http::Method;
use serde_json::json;
use spin_sdk::{
    http::{Request, Response},
    http_component, redis,
};
use uuid::Uuid;

const REDIS_ADDRESS_ENV: &str = "REDIS_URL";

/// A simple Spin HTTP component.
#[http_component]
fn http_gate(req: Request) -> Result<Response> {
    println!("req: {:?}", req);

    let redis_addr = std::env::var(REDIS_ADDRESS_ENV)?;
    println!("redis_addr is: {}", redis_addr);

    let uri = req.uri();
    let path = uri.path().to_owned();

    let mut method = String::new();
    let mut reqdata: Option<String> = None;
    match *req.method() {
        Method::GET => {
            method = "query".to_owned();

            // In query mode: data is the url params
            reqdata = uri.query().map(|query| query.to_string());
        }
        Method::POST => {
            method = "post".to_owned();

            // In post mode: data is the body content of the request
            match req.into_body() {
                Some(body) => {
                    let bo = String::from_utf8_lossy(body.as_ref());
                    reqdata = Some(bo.to_string());
                }
                None => {
                    reqdata = None;
                }
            }
        }
        Method::OPTIONS => {
            return Ok(http::Response::builder()
                .status(200)
                .header("eightfish_version", "0.1")
                .header("Access-Control-Allow-Origin", "*")
                .header("Access-Control-Allow-Methods", "POST, GET, OPTIONS")
                .header("Access-Control-Allow-Headers", "X-PINGOTHER, Content-Type")
                .body(Some("No data".into()))?);
        }
        _ => {
            // handle cases of other directives
            return Ok(http::Response::builder()
                .status(500)
                .body(Some("No data".into()))?);
        }
    };

    // We can do the unified authentication for some actions here
    // depends on path, method, and reqdata
    // XXX:

    // use a unique way to generate a reqid
    let reqid = Uuid::new_v4().simple().to_string();

    let payload = json!({
        "reqid": reqid,
        "reqdata": reqdata,
    });
    println!("payload: {:?}", payload);

    // construct a json, serialize it and send to a redis channel
    // model and action, we can plan a scheme to parse them out
    // here, we just put entire path content to action field, for later cases
    // we can parse it to model and action parts
    let json_to_send = json!({
        "model": path,
        "action": &method,
        "data": payload.to_string().as_bytes().to_vec(),
        "ext": Vec::<u8>::new(),
    });

    if &method == "post" {
        // send to subxt proxy to handle
        _ = redis::publish(
            &redis_addr,
            "spin2proxy",
            &serde_json::to_vec(&json_to_send).unwrap(),
        );
    } else if &method == "query" {
        // send to spin_redis_worker to handle
        _ = redis::publish(
            &redis_addr,
            "proxy2spin",
            &serde_json::to_vec(&json_to_send).unwrap(),
        );
    }

    let mut loop_count = 1;
    loop {
        // loop the redis cache key of this procedure request
        // let result = redis::get(&redis_addr, &format!("cache:{reqid}"))
        // TODO: check the cache:status::{reqid} for the processing status flag
        let status_code = redis::get(&redis_addr, &format!("cache:status:{reqid}"))
            .map_err(|_| anyhow!("Error querying Redis"))?;
        //println!("check result: {:?}", result);

        if status_code.is_empty() {
            // after 20 seconds, timeout
            if loop_count < 2000 {
                // if not get the result, sleep for a little period
                let ten_millis = std::time::Duration::from_millis(10);
                std::thread::sleep(ten_millis);
                loop_count += 1;

                //println!("loop continue {}...", loop_count);
            } else {
                println!("timeout, return 408");
                // timeout handler, use which http status code?
                return Ok(http::Response::builder()
                    .status(408)
                    .header("Access-Control-Allow-Origin", "*")
                    .body(Some("Request Timeout".into()))?);
            }
        } else {
            // Now we get the raw serialized result from worker, we suppose it use
            // JSON spec to serialized it, so we can directly pass it back
            // to user's response body.
            let res_body = redis::get(&redis_addr, &format!("cache:{reqid}"))
                .map_err(|_| anyhow!("Error querying Redis"))?;
            // clear the redis cache key of the worker result
            let _ = redis::del(&redis_addr, &[&format!("cache:status:{reqid}")]);
            let _ = redis::del(&redis_addr, &[&format!("cache:{reqid}")]);

            let status_code = String::from_utf8(status_code).unwrap();
            let status_code = status_code.parse::<u16>().unwrap();
            // jump out this loop, and return the response to user
            return Ok(http::Response::builder()
                .status(status_code)
                .header("eightfish_version", "0.1")
                .header("Access-Control-Allow-Origin", "*")
                .body(Some(Bytes::from(res_body)))?);
        }
    }
}
