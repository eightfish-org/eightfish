#![allow(unreachable_code)]
use futures::StreamExt;
use sp_keyring::AccountKeyring;
//use std::time::Duration;
use subxt::{
    tx::PairSigner,
    OnlineClient,
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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Payload {
    reqid: String,
    reqdata: Vec<(String, String)>,
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

        let api = OnlineClient::<SubstrateConfig>::new().await?;

        let mut of_events = api.events().subscribe().await?.filter_events::<(
            substrate::open_forum_module::events::Action,
            substrate::open_forum_module::events::IndexUpdated)>();

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
                let data = ev.3.clone();
                let time = ev.4;

                let output = InputOutputObject {
                    model,
                    action,
                    data,
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
        let api = OnlineClient::<SubstrateConfig>::new().await.unwrap();

        loop {
            let msg = pubsub_stream.next().await;
            println!("received msg from channel spin2proxy: {:?}", msg);
            let msg_payload: Vec<u8> = msg.unwrap().get_payload()?;

            let msg_obj: InputOutputObject = serde_json::from_slice(&msg_payload).unwrap();

            if &msg_obj.action== "post" {
                // construct act tx
                let tx = substrate::tx()
                    //.openforum()
                    .open_forum_module()
                    .act(msg_obj.model.as_bytes().to_vec(), msg_obj.action.as_bytes().to_vec(), msg_obj.data);

                // Submit the transaction with default params:
                let _hash = api.tx().sign_and_submit_default(&tx, &signer).await.unwrap();

            } else if &msg_obj.action == "update_index" {
                // XXX: here, msg_obj.data contains reqid and reqdata
                // we need to extract the .data to an Payload struct in substrate pallet?
                // call update_index method
                let payload: Payload = serde_json::from_slice(&msg_obj.data).unwrap();
                let (id, hash) = payload.reqdata[0].clone();

                let tx = substrate::tx()
                    //.openforum()
                    .open_forum_module()
                    .update_index(msg_obj.model.as_bytes().to_vec(), payload.reqid.as_bytes().to_vec(), id.as_bytes().to_vec(), hash.as_bytes().to_vec());

                let _hash = api.tx().sign_and_submit_default(&tx, &signer).await.unwrap();

            } else if &msg_obj.action== "check_pair_list" {
                // send rpc request to query the check result

                // XXX: here, msg_obj.data contains reqid and reqdata
                let payload: Payload = serde_json::from_slice(&msg_obj.data).unwrap();

                let params: RpcParams = rpc_params![&msg_obj.model, &payload.reqdata];
                let check_boolean: bool = api
                    .rpc()
                    //.check_pair_list(&msg_obj.model, &payload.reqdata)
                    .request("check_pair_list", params)
                    .await.unwrap();

                let ret_payload = json!({
                    "reqid": payload.reqid,
                    "reqdata": check_boolean.to_string(),
                });

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

                let wasmfile_new_flag = substrate::storage().open_forum_module().wasmfile_new_flag();
                let new_flag: Vec<u8> = api.storage().fetch(&wasmfile_new_flag, None).await;

                // send packet back to the spin runtime
                let output = InputOutputObject {
                    model: msg_obj.model,
                    action: msg_obj.action,
                    data: new_flag,
                    time: 0,
                };
                let output_vec = serde_json::to_vec(&output).unwrap();
                redis_conn1.publish("proxy2upgrade", output_vec).await?;

            } else if &msg_obj.action== "retreive_wasmfile" {

                let wasmfile= substrate::storage().open_forum_module().wasmfile();
                let wasmfile_content: Vec<u8> = api.storage().fetch(&wasmfile, None).await;

                // send packet back to the spin runtime
                let output = InputOutputObject {
                    model: msg_obj.model,
                    action: msg_obj.action,
                    data: wasmfile_content,
                    time: 0,
                };
                let output_vec = serde_json::to_vec(&output).unwrap();
                redis_conn1.publish("proxy2upgrade", output_vec).await?;

            } else if &msg_obj.action== "disable_wasm_upgrade_flag" {

                let tx = substrate::tx()
                    .open_forum_module()
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
