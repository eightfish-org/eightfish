use std::collections::HashMap;
use std::sync::Arc;

use crate::handler::EightFishHandler;
use crate::request::Method;

type InnerRouter = HashMap<Method, Vec<(&'static str, Arc<Box<dyn EightFishHandler>>)>>;

pub struct EightFishRouter {
    router: InnerRouter,
}

impl Default for EightFishRouter {
    fn default() -> Self {
        Self::new()
    }
}

impl EightFishRouter {
    pub fn new() -> EightFishRouter {
        EightFishRouter {
            router: HashMap::new(),
        }
    }

    /// basic router method
    pub fn route<H>(
        &mut self,
        method: Method,
        glob: &'static str,
        handler: H,
    ) -> &mut EightFishRouter
    where
        H: EightFishHandler + 'static,
    {
        self.router
            .entry(method)
            .or_insert(Vec::new())
            .push((glob, Arc::new(Box::new(handler))));
        self
    }

    /// Like route, but specialized to the `Get` method.
    pub fn get<H: EightFishHandler + 'static>(
        &mut self,
        glob: &'static str,
        handler: H,
    ) -> &mut EightFishRouter {
        self.route(Method::Get, glob, handler)
    }

    /// Like route, but specialized to the `Post` method.
    pub fn post<H: EightFishHandler + 'static>(
        &mut self,
        glob: &'static str,
        handler: H,
    ) -> &mut EightFishRouter {
        self.route(Method::Post, glob, handler)
    }

    pub fn into_router(&self) -> &InnerRouter {
        &self.router
    }
}
