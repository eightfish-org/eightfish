use http::HeaderMap;
use serde::de::DeserializeOwned;
use std::collections::HashMap;

#[derive(Eq, Hash, PartialEq, Clone, Copy, Debug)]
pub enum Method {
    Get,
    Post,
    Put,
    Delete,
}

//#[derive(Debug)]
pub struct EightFishRequest {
    method: Method,
    path: String,
    reqid: String,
    headers: HeaderMap,
    proto: Option<String>,
    data: Option<String>,
    ext: HashMap<String, String>,
}

impl EightFishRequest {
    pub fn new(
        method: Method,
        path: String,
        reqid: String,
        headers: HeaderMap,
        proto: Option<String>,
        data: Option<String>,
    ) -> EightFishRequest {
        EightFishRequest {
            method,
            path,
            reqid,
            headers,
            proto,
            data,
            ext: HashMap::new(),
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

    /// get reqid
    pub fn id(&self) -> &String {
        &self.reqid
    }

    /// get headers
    pub fn headers(&self) -> &HeaderMap {
        &self.headers
    }

    /// get proto name
    pub fn proto(&self) -> &Option<String> {
        &self.proto
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

    /// parse urlencoded url or form data
    pub fn parse_urlencoded(
        &self,
    ) -> ::std::result::Result<HashMap<String, String>, anyhow::Error> {
        let mut params: HashMap<String, String> = HashMap::new();

        if let Some(ref data) = self.data {
            let _parse = form_urlencoded::parse(data.as_bytes());
            for pair in _parse {
                let key = pair.0.to_string();
                let val = pair.1.to_string();
                params.insert(key, val);
            }
        }

        Ok(params)
    }

    /// parse json data as a Rust serde struct, may have opion when data is None
    pub fn parse_json<T: DeserializeOwned>(
        &self,
    ) -> ::std::result::Result<Option<T>, anyhow::Error> {
        if let Some(data) = &self.data {
            let parsed: T = serde_json::from_str(data)?;
            Ok(Some(parsed))
        } else {
            Ok(None)
        }
    }

    /// Parse JSON data into any deserializable type (returns error if no data)
    pub fn parse_json_required<T: DeserializeOwned>(
        &self,
    ) -> ::std::result::Result<T, anyhow::Error> {
        if let Some(ref data) = self.data {
            let parsed: T = serde_json::from_str(data)?;
            Ok(parsed)
        } else {
            Err(anyhow::anyhow!("No JSON data available"))
        }
    }

    /// parse json data as a Rust std HashMap
    pub fn parse_json_as_hashmap(
        &self,
    ) -> ::std::result::Result<HashMap<String, String>, anyhow::Error> {
        let mut params: HashMap<String, String> = HashMap::new();
        if let Some(data) = &self.data {
            params = serde_json::from_str(data)?;
        }

        Ok(params)
    }
}
