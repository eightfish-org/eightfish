use eightfish::App as EightFishApp;

mod article;

pub fn start() -> EightFishApp {
    let mut sapp = EightFishApp::new();
    sapp.add_module(Box::new(artile::Article));
    sapp
}


