use std::clone::Clone;
use std::fs::File;
use std::io::Read;
use std::path::Path;
use std::str;
use std::sync::Arc;

use hyper::server::{Handler, Request, Response, Server};
use hyper::status::StatusCode;
use mime_types::Types as MimeTypes;

pub use handler::SapperHandler;
pub use hyper::client::Client;
pub use hyper::header;
pub use hyper::header::Headers;
pub use hyper::mime;
pub use request::SapperRequest;
pub use response::SapperResponse;
pub use router::SapperRouter;
pub use router_m::Router;
pub use typemap::Key;

/// Path parameter type
#[derive(Clone)]
pub struct PathParams;

/// Re-export Status Codes
pub mod status {
    pub use hyper::status::StatusCode;
    pub use hyper::status::StatusCode::*;
}

/// Sapper error enum
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
    fn router(&self, &mut EightFishRouter) -> Result<()>;
}

type GlobalInitClosure = Box<Fn(&mut EightFishRequest) -> Result<()> + 'static + Send + Sync>;

/// EightFish app struct
pub struct EightFishApp {
    // router actually use to recognize
    pub router: Router,
    // if need init something, put them here
    pub init_closure: Option<Arc<GlobalInitClosure>>,
    // 404 not found page
    pub not_found: Option<String>,
}

impl EightFishApp {
    pub fn new() -> EightFishApp {
        EightFishApp {
            router: Router::new(),
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

    // add routers of one module to global router
    pub fn add_module(&mut self, sm: Box<EightFishModule>) -> &mut Self {
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
                let init_closure = self.init_closure.clone();

                self.router.route(
                    method,
                    glob,
                    Arc::new(Box::new(
                        move |req: &mut EightFishRequest| -> Result<EightFishResponse> {
                            if let Some(ref c) = init_closure {
                                c(req)?;
                            }
                            //if let Some(ref armor) = armor {
                            //    armor.before(req)?;
                            //}
                            sm.before(req)?;
                            let mut response: EightFishResponse = handler.handle(req)?;
                            sm.after(req, &mut response)?;
                            //if let Some(ref armor) = armor {
                            //    armor.after(req, &mut response)?;
                            //}
                            Ok(response)
                        },
                    )),
                );
            }
        }

        self
    }

}

impl Handler for EightFishApp {
    /// do actual handling for a request
    fn handle(&self, mut req: EightFishRequest) {
        let path = req.path.clone();

        // pass req to router, execute matched biz handler
        let response = self.router.handle_method(&mut req, &path);
        match response_w {
            Ok(res) => {
            }
            Err(Error::NotFound) => {
            }
            Err(Error::InternalServerError(info)) => {
            }
            Err(_) => {
            }
        }
    }
}

