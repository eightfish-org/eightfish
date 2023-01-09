#![allow(dead_code)]
use spin_sdk::{
    redis_component
};

//mod bizapp;
mod worker;

#[redis_component]
fn on_message(message: Bytes) -> Result<()> {

    // Need optimization, we'd better to keep an Application lifetime instance
    // let aw = worker::Worker::new();
    // aw.mount(bizapp::App::new())?;
    // aw.work(message);

    worker::work(message)?

    Ok(())
}

