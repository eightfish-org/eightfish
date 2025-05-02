use serde::Serialize;
use crate::request::Method;

/// Response status
#[derive(Clone, Debug, Copy)]
pub enum Status {
    Successful,
    Failed,
}

pub trait EightFishModel: Serialize {
    fn model_name(&self) -> String;
    fn id(&self) -> String;
    fn calc_hash(&self) -> String;
}

#[derive(Debug)]
pub struct EightFishResponse<T: EightFishModel + Serialize> {
    status: Status,
    result: Option<Vec<T>>,
    custom_result: Option<String>,
}

// fn do_serialization<T: Serialize>(result: Vec<T>) -> String {
//     serde_json::to_string(&result)
//         .expect("error when do serde_json serialization.")
// }

impl<T: EightFishModel + Serialize> EightFishResponse<T> {
    pub fn new<T: Serialize + EightFishModel>(
        status: Status,
        result: Vec<T>,
    ) -> Self {
        let result = Some(result);
        let custom_result = None;

        EightFishResponse {
            status,
            result,
            custom_result,
        }
    }

    pub fn from_str(status: Status, custom_result: String) -> Self {
        EightFishResponse {
            status,
            result: None,
            custom_result: Some(custom_result)
        }
    }

    /// get response status
    pub fn status(&self) -> Status {
        self.status
    }

    /// set response status
    pub fn set_status(&mut self, status: Status) {
        self.status = status;
    }

    /// get response result
    pub fn result(&self) -> &Option<Vec<T>> {
        &self.result
    }

    /// set result
    pub fn set_result(&mut self, result: Option<Vec<T>>) {
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
}
