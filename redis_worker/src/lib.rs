#![allow(dead_code)]
use spin_sdk::{
    redis_component
};

use myapp::start as myapp_start;
mod worker;

#[redis_component]
fn on_message(message: Bytes) -> Result<()> {
    let app = myapp_start();
    let aw = worker::Worker::mount(app)?;
    aw.work(message)?

    Ok(())
}

