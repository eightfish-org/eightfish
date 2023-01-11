use eightfish::App as EightFishApp;

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


