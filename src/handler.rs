use std::any::Any;

use app::Result;
use request::EightFishRequest as Request;
use response::EightFishResponse as Response;

/// All handlers should implement this Handler trait
pub trait EightFishHandler: Send + Sync + Any {
    fn handle(&self, &mut Request) -> Result<Response>;
}

impl<F> EightFishHandler for F
where
    F: Send + Sync + Any + Fn(&mut Request) -> Result<Response>,
{
    fn handle(&self, req: &mut Request) -> Result<Response> {
        (*self)(req)
    }
}

impl EightFishHandler for Box<EightFishHandler> {
    fn handle(&self, req: &mut Request) -> Result<Response> {
        (**self).handle(req)
    }
}
