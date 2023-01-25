#![allow(dead_code)]
use anyhow::Result;
use spin_sdk::{
    redis_component
};

use myapp::build_app;
mod worker;

#[redis_component]
fn on_message(message: Bytes) -> Result<()> {
    // later put this construtor to outer OnceCell
    let app = build_app();
    let aw = worker::Worker::mount(app)?;

    aw.work(message)?

    Ok(())
}

