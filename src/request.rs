use std::collections::HashMap;

#[derive(Eq, Hash, PartialEq, Clone)]
pub enum Method {
    Get,
    Post,
}

//#[derive(Debug)]
pub struct EightFishRequest {
    method: Method,
    path: String,
    data: Option<String>,
    ext: HashMap<String, String>,
}

impl EightFishRequest {
    pub fn new(method: Method, path: String, data: Option<String>) -> EightFishRequest {
        EightFishRequest {
            method,
            path,
            data,
            ext: HashMap::new(),
        }
    }

    /// get http method
    pub fn method(&self) -> Method {
        self.method.clone()
    }

    /// get http path
    pub fn path(&self) -> &String {
        &self.path
    }

    /// get http data
    pub fn data(&self) -> &Option<String> {
        &self.data
    }

    /// get request struct ext ref
    pub fn ext(&self) -> &HashMap<String, String> {
        &self.ext
    }

    /// get request struct ext mut ref
    pub fn ext_mut(&mut self) -> &mut HashMap<String, String> {
        &mut self.ext
    }

    pub fn parse_urlencoded(&self) -> HashMap<String, String> {
        let mut params: HashMap<String, String> = HashMap::new();

        if self.data.is_some() {
            let _parse = form_urlencoded::parse(self.data.as_ref().unwrap().as_bytes());
            for pair in _parse {
                let key = pair.0.to_string();
                let val = pair.1.to_string();
                params.insert(key, val);
            }
        }

        params
    }
}
