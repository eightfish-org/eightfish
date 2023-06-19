#![allow(dead_code)]
use anyhow::Result;
use bytes::Bytes;
use spin_sdk::redis_component;

use eightfish::{App as EightFishApp, GlobalFilter, Request, Response, Result as EFResult};

mod article;

struct MyGlobalFilter;

impl GlobalFilter for MyGlobalFilter {
    fn before(&self, _req: &mut Request) -> EFResult<()> {
        Ok(())
    }

    fn after(&self, _req: &Request, _res: &mut Response) -> EFResult<()> {
        Ok(())
    }
}

pub fn build_app() -> EightFishApp {
    let mut sapp = EightFishApp::new();
    sapp.add_global_filter(Box::new(MyGlobalFilter))
        .add_module(Box::new(article::ArticleModule));

    sapp
}

/// Main entry
#[redis_component]
fn on_message(message: Bytes) -> Result<()> {
    // later put this construtor to outer OnceCell
    let app = build_app();
    let aw = spin_worker::Worker::mount(app);

    aw.work(message).unwrap();

    Ok(())
}
