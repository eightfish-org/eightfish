use std::any::Any;

use crate::app::Result;
use crate::request::EightFishRequest as Request;
use crate::response::EightFishModel;
use crate::response::EightFishResponse as Response;
use serde::Serialize;

/// All handlers should implement this Handler trait
pub trait EightFishHandler<T>: Send + Sync + Any
where
    T: EightFishModel + Serialize,
{
    fn handle(&self, req: &mut Request) -> Result<Response<T>>;
}

impl<F, T> EightFishHandler<T> for F
where
    F: Send + Sync + Any + Fn(&mut Request) -> Result<Response<T>>,
    T: EightFishModel + Serialize,
{
    fn handle(&self, req: &mut Request) -> Result<Response<T>> {
        (*self)(req)
    }
}

impl<T: EightFishModel + Serialize> EightFishHandler<T> for Box<dyn EightFishHandler<T>> {
    fn handle(&self, req: &mut Request) -> Result<Response<T>> {
        (**self).handle(req)
    }
}
