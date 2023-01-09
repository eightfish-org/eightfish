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
    InvalidConfig,
    InvalidRouterConfig,
    FileNotExist,
    NotFound,
    Unauthorized,                // 401
    Forbidden,                   // 403
    Break(String),               // 400
    InternalServerError(String), // 500
    Found(String),               // 301
    TemporaryRedirect(String),   // 307
    Custom(String),
    CustomHtml(String),
    CustomJson(String),
}

/// EightFish result struct
pub type Result<T> = ::std::result::Result<T, Error>;

/// EightFish module trait
/// 3 methods: before, after, router
pub trait EightFishModule: Sync + Send {
    /// module before filter, will be executed before handler
    fn before(&self, req: &mut EightRequest) -> Result<()> {
        Ok(())
    }

    /// module after filter, will be executed after handler
    fn after(&self, req: &EightFishRequest, res: &mut EightFishResponse) -> Result<()> {
        Ok(())
    }

    /// module router method, used to write router collection of this module here
    fn router(&self, &mut EightFishRouter) -> Result<()>;
}

type GlobalInitClosure = Box<Fn(&mut SapperRequest) -> Result<()> + 'static + Send + Sync>;

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

    // add routers of one module to global routers
    pub fn add_module(&mut self, sm: Box<SapperModule>) -> &mut Self {
        let mut router = SapperRouter::new();
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

                self.routers.route(
                    method,
                    glob,
                    Arc::new(Box::new(
                        move |req: &mut SapperRequest| -> Result<SapperResponse> {
                            if let Some(ref c) = init_closure {
                                c(req)?;
                            }
                            //if let Some(ref armor) = armor {
                            //    armor.before(req)?;
                            //}
                            sm.before(req)?;
                            let mut response: SapperResponse = handler.handle(req)?;
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
    fn handle(&self, req: Request, mut res: Response) {
        let mut sreq = SapperRequest::new(Box::new(req));
        let (path, query) = sreq.uri();

        // pass req to routers, execute matched biz handler
        let response_w = self.routers.handle_method(&mut sreq, &path);
        match response_w {
            Ok(sres) => {
                *res.status_mut() = sres.status();
                match sres.body() {
                    &Some(ref vec) => {
                        for header in sres.headers().iter() {
                            res.headers_mut().set_raw(
                                header.name().to_owned(),
                                vec![header.value_string().as_bytes().to_vec()],
                            );
                        }
                        return res.send(&vec[..]).unwrap();
                    }
                    &None => {
                        return res.send(&"".as_bytes()).unwrap();
                    }
                }
            }
            Err(Error::NotFound) => {
                if self.static_file_service {
                    match simple_file_get(&path) {
                        Ok((file_u8vec, file_mime)) => {
                            res.headers_mut()
                                .set_raw("Content-Type", vec![file_mime.as_bytes().to_vec()]);
                            return res.send(&file_u8vec[..]).unwrap();
                        }
                        Err(_) => {
                            *res.status_mut() = StatusCode::NotFound;
                            return res
                                .send(
                                    self.not_found
                                        .to_owned()
                                        .unwrap_or(String::from("404 Not Found"))
                                        .as_bytes(),
                                )
                                .unwrap();
                        }
                    }
                }

                // return 404 NotFound now
                *res.status_mut() = StatusCode::NotFound;
                return res
                    .send(
                        self.not_found
                            .to_owned()
                            .unwrap_or(String::from("404 Not Found"))
                            .as_bytes(),
                    )
                    .unwrap();
            }
            Err(Error::Break(info)) => {
                *res.status_mut() = StatusCode::BadRequest;
                //return res.send(&"Bad Request".as_bytes()).unwrap();
                return res.send(&info.as_bytes()).unwrap();
            }
            Err(Error::Unauthorized) => {
                *res.status_mut() = StatusCode::Unauthorized;
                return res.send(&"Unauthorized".as_bytes()).unwrap();
            }
            Err(Error::Forbidden) => {
                *res.status_mut() = StatusCode::Forbidden;
                return res.send(&"Forbidden".as_bytes()).unwrap();
            }
            Err(Error::InternalServerError(info)) => {
                *res.status_mut() = StatusCode::InternalServerError;
                //return res.send(&"Internal Server Error".as_bytes()).unwrap();
                return res.send(&info.as_bytes()).unwrap();
            }
            Err(Error::Found(new_uri)) => {
                *res.status_mut() = StatusCode::Found;
                res.headers_mut()
                    .set_raw("Location", vec![new_uri.as_bytes().to_vec()]);
                return res.send(&"Found, Redirect".as_bytes()).unwrap();
            }
            Err(Error::TemporaryRedirect(new_uri)) => {
                *res.status_mut() = StatusCode::TemporaryRedirect;
                res.headers_mut()
                    .set_raw("Location", vec![new_uri.as_bytes().to_vec()]);
                return res.send(&"Temporary Redirect".as_bytes()).unwrap();
            }
            Err(Error::Custom(ustr)) => {
                *res.status_mut() = StatusCode::Ok;
                return res.send(&ustr.as_bytes()).unwrap();
            }
            Err(Error::CustomHtml(html_str)) => {
                *res.status_mut() = StatusCode::Ok;
                res.headers_mut()
                    .set_raw("Content-Type", vec!["text/html".as_bytes().to_vec()]);
                return res.send(&html_str.as_bytes()).unwrap();
            }
            Err(Error::CustomJson(json_str)) => {
                *res.status_mut() = StatusCode::Ok;
                res.headers_mut().set_raw(
                    "Content-Type",
                    vec!["application/x-javascript".as_bytes().to_vec()],
                );
                return res.send(&json_str.as_bytes()).unwrap();
            }
            Err(_) => {
                *res.status_mut() = StatusCode::InternalServerError;
                return res.send(&"InternalServerError".as_bytes()).unwrap();
            }
        }
    }
}

