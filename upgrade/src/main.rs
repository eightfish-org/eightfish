use tokio::fs::File;
use tokio::io::AsyncWriteExt;
use tokio::time;
use tokio_stream::StreamExt;

use redis::AsyncCommands;

use serde::{Deserialize, Serialize};
use serde_json::json;

const UPGRADE2PROXYCHANNEL: &str = "spin2proxy";
const PROXY2UPGRADECHANNEL: &str = "proxy2upgrade";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InputOutputObject {
    model: String,
    action: String,
    data: Vec<u8>,
    ext: Vec<u8>,
}

#[tokio::main]
async fn main() {
    let mut interval = time::interval(time::Duration::from_secs(15));
    loop {
        interval.tick().await;
        do_task().await;
    }
}

async fn do_task() {
    // load the dotenv config
    let key = "WASMFILE";
    let wasmfile_path: String = dotenv::var(key).unwrap();
    let key = "REDISADDR";
    let redis_addr: String = dotenv::var(key).unwrap();

    // create redis connections
    let redis_client = redis::Client::open(redis_addr).unwrap();
    let mut redis_conn = redis_client.get_async_connection().await.unwrap();
    let mut pubsub_conn = redis_client
        .get_async_connection()
        .await
        .unwrap()
        .into_pubsub();
    pubsub_conn.subscribe(PROXY2UPGRADECHANNEL).await.unwrap();
    let mut pubsub_stream = pubsub_conn.on_message();

    // check whether has a new wasm content, by sending msg to redis channel
    let json_to_send = json!({
        "model": "upgrade",
        "action": "check_new_version_wasmfile",
        "data": Vec::<u8>::new(),
        "ext": Vec::<u8>::new(),
    });
    let trans_string = json_to_send.to_string();
    let _: Result<String, redis::RedisError> =
        redis_conn.publish(UPGRADE2PROXYCHANNEL, trans_string).await;

    // wait for the checking response in asynchronization
    let msg = pubsub_stream.next().await;
    // println!("received msg from proxy: {:?}", msg);
    let msg_payload: Vec<u8> = msg.unwrap().get_payload().unwrap();
    let msg_obj: InputOutputObject = serde_json::from_slice(&msg_payload).unwrap();
    println!(
        "received msg from proxy: {}, {}, {}, {}",
        msg_obj.model,
        msg_obj.action,
        if msg_obj.data.len() == 5 {
            "false"
        } else if msg_obj.data.len() == 4 {
            "true"
        } else {
            ""
        },
        msg_obj.ext.len()
    );

    if &msg_obj.action == "check_new_version_wasmfile" && &msg_obj.data == b"true" {
        // do the retreiving content action
        let json_to_send = json!({
            "model": "upgrade",
            "action": "retreive_wasmfile",
            "data": Vec::<u8>::new(),
            "ext": Vec::<u8>::new(),
        });
        let trans_string = json_to_send.to_string();
        let _: Result<String, redis::RedisError> =
            redis_conn.publish(UPGRADE2PROXYCHANNEL, trans_string).await;

        let msg = pubsub_stream.next().await;
        let msg_payload: Vec<u8> = msg.unwrap().get_payload().unwrap();
        let msg_obj: InputOutputObject = serde_json::from_slice(&msg_payload).unwrap();
        println!(
            "received msg from proxy: {}, {}, {}, {}",
            msg_obj.model,
            msg_obj.action,
            msg_obj.data.len(),
            msg_obj.ext.len()
        );
        if &msg_obj.action == "retreive_wasmfile" && !msg_obj.data.is_empty() {
            // after getting the wasm blob data, write it to the destination file, by the path got from
            // the config file
            let mut file = File::create(wasmfile_path).await.unwrap();
            file.write_all(&msg_obj.data).await.unwrap();
            println!("write file successfully!");

            // disable the wasm upgrade flag
            let json_to_send = json!({
                "model": "upgrade",
                "action": "disable_wasm_upgrade_flag",
                "data": Vec::<u8>::new(),
                "ext": Vec::<u8>::new(),
            });
            let trans_string = json_to_send.to_string();
            let _: Result<String, redis::RedisError> =
                redis_conn.publish(UPGRADE2PROXYCHANNEL, trans_string).await;

            // currently, we don't process the returned event value of the disable_wasm_upgrade_flag call

            // DONE
        }
    } else {
        // do nothing, only log it
    }
}
