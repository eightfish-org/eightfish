mod handler;
mod recognizer;
mod router;
mod router_m;
mod request;
mod response;
mod app;

//pub use app::Client;
/// PathParams is the parameter type refering the parameters collected in url
pub use app::PathParams;
pub use app::EightFishApp as App;
pub use app::GlobalFilter;
pub use app::EightFishHandler as Handler;
pub use app::EightFishModule as Module;
pub use app::EightFishRouter as Router;
pub use app::{Error, Key, Result};
pub use request::{EightFishRequest as Request, Method};
pub use response::{EightFishResponse as Response, Status, Info};

pub use recognizer::Params as RecognizerParams;
