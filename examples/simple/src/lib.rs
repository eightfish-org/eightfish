#![allow(dead_code)]
use anyhow::Result;
use bytes::Bytes;
use spin_sdk::{redis_component};

use eightfish::{
    App as EightFishApp,
    GlobalFilter,
};

mod article;

struct MyGlobalFilter;

impl GlobalFilter for MyGlobalFilter {
    fn before(&self, &mut Request) -> Result<()> {

    }

    fn after(&self, &Request, &mut Response) -> Result<()> {

    }
}

pub fn build_app() -> EightFishApp {
    let mut sapp = EightFishApp::new();
    sapp.add_global_filter(MyGlobalFilter)
        .add_module(Box::new(artile::ArticleModule))
}

/// Main entry
#[redis_component]
fn on_message(message: Bytes) -> Result<()> {
    // later put this construtor to outer OnceCell
    let app = build_app();
    let aw = spin_worker::Worker::mount(app)?;

    aw.work(message)?

    Ok(())
}

