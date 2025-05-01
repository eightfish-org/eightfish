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
pub struct EightFishResponse {
    status: Status,
    method: Method,
    model_name: String,
    pair_list: Option<Vec<(String, String)>>,
    result: Option<String>,
}

fn do_serialization<T: Serialize>(results: Vec<T>) -> String {
    serde_json::to_string(&results).expect("error when do serde_json serialization.")
}

impl EightFishResponse {
    pub fn new<T: Serialize + EightFishModel>(
        status: Status,
        method: Method,
        results: Vec<T>,
    ) -> EightFishResponse {
        let model_name = T::model_name();
        let pair_list;
        let result;

        if results.is_empty() {
            pair_list = None;
            result = None;
        } else {
            let a_pair_list = results
                .iter()
                .map(|obj| (obj.id(), obj.calc_hash()))
                .collect();
            pair_list = Some(a_pair_list);
            let output = do_serialization(results);
            result = Some(output);
        }

        EightFishResponse {
            status,
            method,
            model_name,
            pair_list,
            result,
        }
    }

    pub fn from_str(status: Status, method: Method, results: String) -> EightFishResponse {
        EightFishResponse {
            status,
            method,
            model_name: "from_str".to_string(),
            pair_list: None,
            result: Some(results),
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

    /// get response model_name field
    pub fn model_name(&self) -> &str {
        &self.model_name
    }

    /// get response pair_list
    pub fn pair_list(&self) -> &Option<Vec<(String, String)>> {
        &self.pair_list
    }

    /// get response result
    pub fn result(&self) -> &Option<String> {
        &self.result
    }

    /// set result
    pub fn set_result(&mut self, result: Option<String>) {
        self.result = result;
    }
}

#[macro_export]
macro_rules! res_ok {
    ($results:expr) => {
        // assume in EightFish handler, the req object is named in the input parameter
        Ok(EightFishResponse::new(Status::Successful, req.method(), $results))
    };
}
