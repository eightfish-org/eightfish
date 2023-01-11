use eightfish::App as EightFishApp;

mod article;

struct MyGlobalFilter;

impl GlobalFilter for MyGlobalFilter {
    fn before(&self, &mut EightFishRequest) -> Result<()> {

    }

    fn after(&self, &EightFishRequest, &mut EightFishResponse) -> Result<()> {

    }
}


pub fn start() -> EightFishApp {
    let mut sapp = EightFishApp::new();
    sapp.add_global_filter(MyGlobalFilter)
        .add_module(Box::new(artile::ArticleModule))
}


