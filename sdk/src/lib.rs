#![allow(dead_code)]
mod app;
mod handler;
mod recognizer;
mod request;
mod response;
mod router;
mod router_m;

pub use app::EightFishApp as App;
pub use app::EightFishHandlerCRUD as HandlerCRUD;
pub use app::EightFishModule as Module;
pub use app::EightFishRouter as Router;
pub use app::GlobalFilter;
pub use app::{Error, Result};
pub use handler::EightFishHandler as Handler;
pub use request::{EightFishRequest as Request, Method};
pub use response::{EightFishModel, EightFishResponse as Response, Info, Status};

pub use recognizer::Params as RecognizerParams;
