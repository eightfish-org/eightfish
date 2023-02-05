mod app;
mod handler;
mod recognizer;
mod request;
mod response;
mod router;
mod router_m;

//pub use app::Client;
pub use app::EightFishApp as App;
pub use app::EightFishModule as Module;
pub use app::EightFishRouter as Router;
pub use app::GlobalFilter;
/// PathParams is the parameter type refering the parameters collected in url
pub use app::PathParams;
pub use app::{Error, Key, Result};
pub use handler::EightFishHandler as Handler;
pub use request::{EightFishRequest as Request, Method};
pub use response::{EightFishModel, EightFishResponse as Response, Info, Status};

pub use recognizer::Params as RecognizerParams;
