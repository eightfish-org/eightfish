use crate::response::EightFishModel;
use serde::Serialize;
use std::collections::HashMap;
use std::sync::Arc;

use crate::handler::EightFishHandler;
use crate::request::Method;

type InnerRouter<T: EightFishModel + Serialize> =
    HashMap<Method, Vec<(&'static str, Arc<Box<dyn EightFishHandler<T>>>)>>;

pub struct EightFishRouter<T>
where
    T: EightFishModel + Serialize,
{
    router: InnerRouter<T>,
}

impl<T> Default for EightFishRouter<T>
where
    T: EightFishModel + Serialize,
{
    fn default() -> Self {
        Self::new()
    }
}

impl<T> EightFishRouter<T>
where
    T: EightFishModel + Serialize,
{
    pub fn new() -> EightFishRouter<T> {
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
    ) -> &mut EightFishRouter<T>
    where
        H: EightFishHandler<T> + 'static,
    {
        self.router
            .entry(method)
            .or_insert(Vec::new())
            .push((glob, Arc::new(Box::new(handler))));
        self
    }

    /// Like route, but specialized to the `Get` method.
    pub fn get<H: EightFishHandler<T> + 'static>(
        &mut self,
        glob: &'static str,
        handler: H,
    ) -> &mut EightFishRouter<T> {
        self.route(Method::Get, glob, handler)
    }

    /// Like route, but specialized to the `Post` method.
    pub fn post<H: EightFishHandler<T> + 'static>(
        &mut self,
        glob: &'static str,
        handler: H,
    ) -> &mut EightFishRouter<T> {
        self.route(Method::Post, glob, handler)
    }

    pub fn put<H: EightFishHandler<T> + 'static>(
        &mut self,
        glob: &'static str,
        handler: H,
    ) -> &mut EightFishRouter<T> {
        self.route(Method::Put, glob, handler)
    }

    pub fn delete<H: EightFishHandler<T> + 'static>(
        &mut self,
        glob: &'static str,
        handler: H,
    ) -> &mut EightFishRouter<T> {
        self.route(Method::Delete, glob, handler)
    }

    pub fn into_router(&self) -> &InnerRouter<T> {
        &self.router
    }
}
