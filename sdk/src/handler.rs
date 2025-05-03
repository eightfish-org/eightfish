use std::any::Any;

use crate::app::Result;
use crate::request::EightFishRequest as Request;
use crate::response::EightFishResponse as Response;

/// All handlers should implement this Handler trait
pub trait EightFishHandler: Send + Sync + Any {
    fn handle<T: EightFishModel + Serialize>(&self, req: &mut Request) -> Result<Response<T>>;
}

impl<F, T> EightFishHandler for F
where
    F: Send + Sync + Any + Fn(&mut Request) -> Result<Response<T>>,
    T: EightFishModel + Serialize,
{
    fn handle((&self, req: &mut Request) -> Result<Response<T>> {
        (*self)(req)
    }
}

impl EightFishHandler for Box<dyn EightFishHandler> {
    fn handle<T: EightFishModel + Serialize>(&self, req: &mut Request) -> Result<Response<T>> {
        (**self).handle(req)
    }
}
