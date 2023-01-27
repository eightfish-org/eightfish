#![allow(dead_code)]
use bytes::Bytes;
use spin_sdk::{redis_component};

use eightfish::{
    App as EightFishApp,
    GlobalFilter,
    Request,
    Response,
    Result,
};

mod article;

struct MyGlobalFilter;

impl GlobalFilter for MyGlobalFilter {
    fn before(&self, req: &mut Request) -> Result<()> {
        Ok(())
    }

    fn after(&self, req: &Request, res: &mut Response) -> Result<()> {
        Ok(())
    }
}

pub fn build_app() -> EightFishApp {
    let mut sapp = EightFishApp::new();
    sapp.add_global_filter(MyGlobalFilter)
        .add_module(Box::new(article::ArticleModule))
}

/// Main entry
#[redis_component]
fn on_message(message: Bytes) -> anyhow::Result<()> {
    // later put this construtor to outer OnceCell
    let app = build_app();
    let aw = spin_worker::Worker::mount(app);

    aw.work(message).unwrap();

    Ok(())
}

