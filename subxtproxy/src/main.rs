#![allow(unreachable_code)]
use std::fmt;
use futures::StreamExt;
use sp_keyring::AccountKeyring;
//use std::time::Duration;
use subxt::{
    tx::PairSigner,
    OnlineClient,
    PolkadotConfig,
    SubstrateConfig,
};
use subxt::rpc::{ rpc_params, RpcParams };

use serde::{Serialize, Deserialize};
use serde_json::json;

//use futures_util::StreamExt as _;
//use futures::StreamExt as _;
use redis::AsyncCommands;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InputOutputObject {
    model: String,
    action: String,
    data: Vec<u8>,
    time: u64,
}


type PairList = Vec<(String, String)>;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Payload {
    reqid: String,
    reqdata: Option<PairList>,
}

/*
impl fmt::Display for PairList {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut output = String::new();
        for (id, hash) in self {
            output.push_str(&id);
            output.push_str(",");
            output.push_str(&hash);
            output.push_str(",");
        }
        if output.len() > 0 {
            output = output.truncate(output.len() - 1);
        }
        write!(f, "{}", output)
    }
}
*/

fn to_hex(bytes: impl AsRef<[u8]>) -> String {
    format!("0x{}", hex::encode(bytes.as_ref()))
}



#[subxt::subxt(runtime_metadata_path = "metadata.scale")]
pub mod substrate {}

/// Subscribe to all events, and then manually look through them and
/// pluck out the events that we care about.
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt::init();

    let redis_client = redis::Client::open("redis://127.0.0.1/").unwrap();
    let mut redis_conn = redis_client.get_async_connection().await?;
    let mut redis_conn1 = redis_client.get_async_connection().await?;
    let mut pubsub_conn = redis_client.get_async_connection().await?.into_pubsub();

    let task_subxt = tokio::task::spawn(async move {

        //let api = OnlineClient::<SubstrateConfig>::new().await?;
        let api = OnlineClient::<PolkadotConfig>::new().await?;

        let mut of_events = api.events().subscribe().await?.filter_events::<(
            substrate::eight_fish_module::events::Action,
            substrate::eight_fish_module::events::IndexUpdated)>();

        while let Some(evt) = of_events.next().await {
            let event_details = evt?;

            let block_hash = event_details.block_hash;
            let event = event_details.event;
            println!("Event at {:?}:", block_hash);

            if let (Some(ev), _) = &event {
                println!("  Action event: {ev:?}");

                let model = String::from_utf8(ev.1.clone()).unwrap();
                let action = String::from_utf8(ev.2.clone()).unwrap();
                let data = ev.3.clone();
                let time = ev.4;

                let output = InputOutputObject {
                    model,
                    action,
                    data,
                    time,
                };

                let output_vec = serde_json::to_vec (&output).unwrap();
                let _: Result<String, redis::RedisError> = redis_conn.publish("proxy2spin", output_vec).await;

            }
            if let (_, Some(ev)) = &event {
                println!("  IndexUpdated event: {ev:?}");

                let model = String::from_utf8(ev.1.clone()).unwrap();
                let action = String::from_utf8(ev.2.clone()).unwrap();
                let data = String::from_utf8(ev.3.clone()).unwrap();
                let time = ev.4;

                let v: Vec<&str> = data.split(':').collect();
                println!("IndexUpdated event: v: {:?}", v);
                let reqid = &v[0];
                let id = &v[1];

                let payload = json!({
                    "reqid": reqid,
                    "reqdata": Some(id),
                });

                let output = InputOutputObject {
                    model,
                    action,
                    data: payload.to_string().as_bytes().to_vec(),
                    time,
                };

                let output_vec = serde_json::to_vec(&output).unwrap();
                let _: Result<String, redis::RedisError> = redis_conn.publish("proxy2spin", output_vec).await;
            }

        }

        Ok::<(), subxt::Error>(())
    });
    

    // ==============================
    // redis listener part
    //
    let task_redis = tokio::task::spawn(async move {
        pubsub_conn.subscribe("spin2proxy").await?;
        let mut pubsub_stream = pubsub_conn.on_message();

        // Get a instance of subxt to send transactions to substrate
        let signer = PairSigner::new(AccountKeyring::Alice.pair());
        let api = OnlineClient::<PolkadotConfig>::new().await.unwrap();

        loop {
            let msg = pubsub_stream.next().await;
            println!("received msg from channel spin2proxy: {:?}", msg);
            let msg_payload: Vec<u8> = msg.unwrap().get_payload()?;

            let msg_obj: InputOutputObject = serde_json::from_slice(&msg_payload).unwrap();

            if &msg_obj.action== "post" {
                println!("from redis: post: {:?}", msg_obj);
                // construct act tx
                let tx = substrate::tx()
                    .eight_fish_module()
                    .act(msg_obj.model.as_bytes().to_vec(), 
                         msg_obj.action.as_bytes().to_vec(), 
                         msg_obj.data);

                // Submit the transaction with default params:
                let _hash = api.tx().sign_and_submit_default(&tx, &signer).await.unwrap();
            } else if &msg_obj.action == "update_index" {
                println!("from redis: update_index: {:?}", msg_obj);
                // XXX: here, msg_obj.data contains reqid and reqdata
                // we need to extract the .data to an Payload struct in substrate pallet?
                // call update_index method
                let payload: Payload = serde_json::from_slice(&msg_obj.data).unwrap();
                println!("from redis: update_index: payload: {:?}", payload);
                let reqid = payload.reqid.clone();
                let data = payload.reqdata.clone().unwrap();
                let (id, hash) = &data[0];

                let tx = substrate::tx()
                    .eight_fish_module()
                    .update_index(msg_obj.model.as_bytes().to_vec(), 
                                  reqid.as_bytes().to_vec(), 
                                  id.as_bytes().to_vec(), 
                                  hash.as_bytes().to_vec());

                let _hash = api.tx().sign_and_submit_default(&tx, &signer).await.unwrap();

            } else if &msg_obj.action== "check_pair_list" {
                println!("from redis: check_pair_list: {:?}", msg_obj);
                // send rpc request to query the check result
                // XXX: here, msg_obj.data contains reqid and reqdata
                let payload: Payload = serde_json::from_slice(&msg_obj.data).unwrap();
                println!("from redis: check_pair_list: payload: {:?}", payload);
                let model = msg_obj.model.clone();
                let reqdata = payload.reqdata.clone().unwrap();
                //println!("check_pair_list: reqdata.to_string(): {:?}", reqdata.to_string());
                /*
                let mut output = String::new();
                for (id, hash) in reqdata {
                    output.push_str(&id);
                    output.push_str(",");
                    output.push_str(&hash);
                    output.push_str(",");
                }
                if output.len() > 0 {
                    output.truncate(output.len() - 1);
                }
                */
                let pair_list: PairList = reqdata.iter().map(|(id, hash)| (to_hex(id.as_bytes()), hash.clone())).collect();

                let params: RpcParams = rpc_params![to_hex(model.as_bytes()), pair_list];
                //let mut params = RpcParams::new();
                //params.push(hex::encode(&msg_obj.model.clone())).unwrap();
                //params.push(hex::encode(&output)).unwrap();

                let check_boolean: bool = api
                    .rpc()
                    //.eightfish_checkPairList(model, reqdata)
                    .request("eightfish_checkPairList", params)
                    .await.unwrap();

                let ret_payload = json!({
                    "reqid": payload.reqid,
                    "reqdata": Some(check_boolean.to_string()),
                });
                println!("from redis: check_pair_list: ret_payload: {:?}", ret_payload);

                // send packet back to the spin runtime
                let output = InputOutputObject {
                    model: msg_obj.model,
                    action: msg_obj.action,
                    data: ret_payload.to_string().as_bytes().to_vec(),
                    time: 0,
                };
                let output_string = serde_json::to_vec(&output).unwrap();
                redis_conn1.publish("proxy2spin", output_string).await?;

            } else if &msg_obj.action== "check_new_version_wasmfile" {
                println!("from redis: check_new_version_wasmfile: {:?}", msg_obj);

                let wasmfile_new_flag = substrate::storage().eight_fish_module().wasm_file_new_flag();
                let new_flag: Option<bool> = api.storage().fetch(&wasmfile_new_flag, None).await.unwrap();
                if let Some(flag) = new_flag {
                    // send packet back to the spin runtime
                    let output = InputOutputObject {
                        model: msg_obj.model,
                        action: msg_obj.action,
                        data: flag.to_string().as_bytes().to_vec(),
                        time: 0,
                    };
                    let output_vec = serde_json::to_vec(&output).unwrap();
                    redis_conn1.publish("proxy2upgrade", output_vec).await?;
                } else {
                    println!("check_new_version_wasmfile error, return None");
                }

            } else if &msg_obj.action== "retreive_wasmfile" {
                println!("from redis: retreive_wasmfile: {:?}", msg_obj);

                let wasmfile= substrate::storage().eight_fish_module().wasm_file();
                let wasmfile_content: Option<Vec<u8>> = api.storage().fetch(&wasmfile, None).await.unwrap();

                if let Some(wasmfile) = wasmfile_content {
                    // send packet back to the spin runtime
                    let output = InputOutputObject {
                        model: msg_obj.model,
                        action: msg_obj.action,
                        data: wasmfile,
                        time: 0,
                    };
                    let output_vec = serde_json::to_vec(&output).unwrap();
                    redis_conn1.publish("proxy2upgrade", output_vec).await?;
                } else {
                    println!("retreive_wasmfile error, return None");
                }

            } else if &msg_obj.action== "disable_wasm_upgrade_flag" {
                println!("from redis: disable_wasm_upgrade_flag: {:?}", msg_obj);

                let tx = substrate::tx()
                    .eight_fish_module()
                    .disable_wasm_upgrade_flag();

                let _hash = api.tx().sign_and_submit_default(&tx, &signer).await.unwrap();

                // we can also use the then_watch api to wait for the return of the call
                // get the event and check the success of it
                //.sign_and_submit_then_watch_default(&_tx, &signer)
            }
        }

        Ok::<(), redis::RedisError>(())
    });

    let (_, _) = tokio::join!(task_subxt, task_redis);
    
    Ok(())
}
