use std::clone::Clone;
use std::sync::Arc;

pub use crate::handler::EightFishHandler;
pub use crate::request::EightFishRequest;
pub use crate::response::EightFishResponse;
pub use crate::router::EightFishRouter;
pub use crate::router_m::Router;

/// EightFish Error
#[derive(Debug, PartialEq, Clone)]
pub enum Error {
    //InvalidConfig,
    //InvalidRouterConfig,
    //FileNotExist,
    NotFound,
    //Unauthorized,                // 401
    //Forbidden,                   // 403
    //Break(String),               // 400
    InternalServerError(String), // 500
    //Found(String),               // 301
    //TemporaryRedirect(String),   // 307
    //Custom(String),
    //CustomHtml(String),
    //CustomJson(String),
}

/// EightFish result struct
pub type Result<T> = ::std::result::Result<T, Error>;

/// GlobalFilter trait, used to place global `before` and `after` middlewares
pub trait GlobalFilter {
    fn before(&self, req: &mut EightFishRequest) -> Result<()>;
    fn after(&self, req: &EightFishRequest, res: &mut EightFishResponse) -> Result<()>;
}
type GlobalFilterType = Box<dyn GlobalFilter + 'static + Send + Sync>;
type GlobalInitClosure = Box<dyn Fn(&mut EightFishRequest) -> Result<()> + 'static + Send + Sync>;

/// EightFish module trait
/// 3 methods: before, after, router
pub trait EightFishModule: Sync + Send {
    /// module before filter, will be executed before handler
    fn before(&self, req: &mut EightFishRequest) -> Result<()> {
        Ok(())
    }

    /// module after filter, will be executed after handler
    fn after(&self, req: &EightFishRequest, res: &mut EightFishResponse) -> Result<()> {
        Ok(())
    }

    /// module router method, used to write router collection of this module here
    fn router(&self, router: &mut EightFishRouter) -> Result<()>;
}


/// EightFish app struct
pub struct EightFishApp {
    // router actually use to recognize
    pub router: Router,
    // global filter if exists
    pub global_filter: Option<Arc<GlobalFilterType>>,
    // if need init something, put them here
    pub init_closure: Option<Arc<GlobalInitClosure>>,
    // 404 not found page
    pub not_found: Option<String>,
}

impl EightFishApp {
    pub fn new() -> EightFishApp {
        EightFishApp {
            router: Router::new(),
            global_filter: None,
            init_closure: None,
            not_found: None,
        }
    }

    // init something, usually in global scope
    pub fn init_global(&mut self, clos: GlobalInitClosure) -> &mut Self {
        self.init_closure = Some(Arc::new(clos));
        self
    }

    // define 404 not found page here
    pub fn not_found_page(&mut self, page: String) -> &mut Self {
        self.not_found = Some(page);
        self
    }

    // add global filter
    pub fn add_global_filter(&mut self, w: GlobalFilterType) -> &mut Self {
        self.global_filter = Some(Arc::new(w));
        self
    }

    // add routers of one module to global router
    pub fn add_module(&mut self, sm: Box<dyn EightFishModule>) -> &mut Self {
        let mut router = EightFishRouter::new();
        // get the sm router
        sm.router(&mut router).unwrap();
        let sm = Arc::new(sm);

        for (method, handler_vec) in router.into_router() {
            // add to wrapped router
            for &(glob, ref handler) in handler_vec.iter() {
                let method = method.clone();
                let glob = glob.clone();
                let handler = handler.clone();
                let sm = sm.clone();
                let global_filter = self.global_filter.clone();
                let init_closure = self.init_closure.clone();

                self.router.route(
                    method,
                    glob,
                    Arc::new(Box::new(
                        move |req: &mut EightFishRequest| -> Result<EightFishResponse> {
                            if let Some(ref c) = init_closure {
                                c(req)?;
                            }
                            if let Some(ref global_filter) = global_filter {
                                global_filter.before(req)?;
                            }
                            sm.before(req)?;
                            let mut response: EightFishResponse = handler.handle(req)?;
                            sm.after(req, &mut response)?;
                            if let Some(ref global_filter) = global_filter {
                                global_filter.after(req, &mut response)?;
                            }
                            Ok(response)
                        },
                    )),
                );
            }
        }

        self
    }

}

impl EightFishHandler for EightFishApp {
    /// do actual handling for a request
    fn handle(&self, mut req: &mut EightFishRequest) -> Result<EightFishResponse> {
        let path = req.path().clone();

        // pass req to router, execute matched biz handler
        let response = self.router.handle_method(&mut req, &path);
        response
/*            
        match response {
            Ok(res) => {
                Ok(res)
            }
            Err(Error::NotFound) => {
            }
            Err(Error::InternalServerError(info)) => {
            }
            Err(_) => {
            }
        }
*/

    }
}

