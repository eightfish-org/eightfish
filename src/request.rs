use typemap::TypeMap;

pub struct Method {
    Get,
    Post,
}


pub struct EightFishRequest {
    method: Method,
    path: String,
    data: Option<String>,
    ext: TypeMap,
}

impl EightFishRequest {
    pub fn new(method: Method, path: String, data: Option<String>) -> EightFishRequest {
        EightFishRequest {
            method: method,
            path: path,
            data: data,
            ext: TypeMap::new(),
        }
    }

    /// get http method
    pub fn method(&self) -> Method {
        self.method
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
    pub fn ext(&self) -> &TypeMap {
        &self.ext
    }

    /// get request struct ext mut ref
    pub fn ext_mut(&mut self) -> &mut TypeMap {
        &mut self.ext
    }
}
