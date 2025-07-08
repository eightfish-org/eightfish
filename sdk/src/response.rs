use http::header::{HeaderMap, HeaderName, HeaderValue};
use serde::Serialize;
use serde_json::Value;
// use std::collections::HashMap;

pub use http::status::StatusCode;
// /// Response status
// #[derive(Clone, Debug, Copy)]
// pub enum Status {
//     Successful,
//     Failed,
// }

pub trait EightFishModel: Serialize {
    fn model_name(&self) -> String;
    fn id(&self) -> String;
    fn calc_hash(&self) -> String;
}

#[derive(Debug)]
pub struct EightFishResponse {
    status: StatusCode,
    headers: Option<HeaderMap>,
    result: Option<String>,
    custom_result: Option<String>,
    model_name: Option<String>,
    pair_list: Option<Vec<(String, String)>>,
}

impl EightFishResponse {
    pub fn new_check<T: EightFishModel + Serialize>(result: Vec<T>) -> Self {
        let custom_result = None;

        let model_name = if result.is_empty() {
            None
        } else {
            Some(result[0].model_name())
        };
        let pair_list = result
            .iter()
            .map(|elem| (elem.id(), elem.calc_hash()))
            .collect();

        let data = serde_json::to_string(&result).expect("error when do serde_json serialization.");

        let mut headers = HeaderMap::new();
        let json_header = "application/json"
            .parse()
            .expect("str parsing error in Response new.");
        headers.insert(http::header::CONTENT_TYPE, json_header);

        EightFishResponse {
            status: StatusCode::OK,
            headers: Some(headers),
            result: Some(data),
            custom_result,
            model_name,
            pair_list: Some(pair_list),
        }
    }

    pub fn new_uncheck(status: StatusCode, json_result: Value) -> Self {
        let jsonstr = json_result.to_string();

        let mut headers = HeaderMap::new();
        let json_header = "application/json"
            .parse()
            .expect("str parsing error in Response from_failed.");
        headers.insert(http::header::CONTENT_TYPE, json_header);

        EightFishResponse {
            status,
            headers: Some(headers),
            result: None,
            custom_result: Some(jsonstr),
            model_name: None,
            pair_list: None,
        }
    }

    pub fn from_str_uncheck(status: StatusCode, custom_result: String) -> Self {
        let mut headers = HeaderMap::new();
        let header = "text/plain"
            .parse()
            .expect("str parsing error in Response from_failed.");
        headers.insert(http::header::CONTENT_TYPE, header);

        EightFishResponse {
            status,
            headers: Some(headers),
            result: None,
            custom_result: Some(custom_result),
            model_name: None,
            pair_list: None,
        }
    }

    pub fn from_html_uncheck(status: StatusCode, custom_result: String) -> Self {
        let mut headers = HeaderMap::new();
        let header = "text/html"
            .parse()
            .expect("str parsing error in Response from_failed.");
        headers.insert(http::header::CONTENT_TYPE, header);

        EightFishResponse {
            status,
            headers: Some(headers),
            result: None,
            custom_result: Some(custom_result),
            model_name: None,
            pair_list: None,
        }
    }

    pub fn set_header(&mut self, name: HeaderName, value: HeaderValue) {
        // Get or create HeaderMap
        let headers = self.headers.get_or_insert_with(HeaderMap::new);

        // Insert the header
        headers.insert(name, value);
    }

    /// get response status
    pub fn status_code(&self) -> StatusCode {
        self.status
    }

    /// set response status
    pub fn set_status_code(&mut self, status: StatusCode) {
        self.status = status;
    }

    /// get response headers
    pub fn headers(&self) -> &Option<HeaderMap> {
        &self.headers
    }

    /// set response status
    pub fn set_headers(&mut self, headers: Option<HeaderMap>) {
        self.headers = headers;
    }

    /// get response result
    pub fn result(&self) -> &Option<String> {
        &self.result
    }

    /// set result
    pub fn set_result(&mut self, result: Option<String>) {
        self.result = result;
    }

    /// get response customized result
    pub fn custom_result(&self) -> &Option<String> {
        &self.custom_result
    }

    /// set custom result
    pub fn set_custom_result(&mut self, custom_result: Option<String>) {
        self.custom_result = custom_result;
    }

    // get model name
    pub fn model_name(&self) -> &Option<String> {
        &self.model_name
    }

    /// get pair list
    pub fn pair_list(&self) -> &Option<Vec<(String, String)>> {
        &self.pair_list
    }
}
