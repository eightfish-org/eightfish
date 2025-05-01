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
}

// fn do_serialization<T: Serialize>(result: Vec<T>) -> String {
//     serde_json::to_string(&result)
//         .expect("error when do serde_json serialization.")
// }

impl<T: EightFishModel + Serialize> EightFishResponse<T> {
    pub fn new<T: Serialize + EightFishModel>(
        status: Status,
        result: Vec<T>,
    ) -> EightFishResponse {
        let model_name = T::model_name();

        // let output = do_serialization(result);
        let result = Some(result);

        EightFishResponse {
            status,
            result,
        }
    }

    pub fn from_str(status: Status, result: String) -> EightFishResponse {
        EightFishResponse {
            status,
            result: Some(result),
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
}

#[macro_export]
macro_rules! res_ok {
    ($results:expr) => {
        // assume in EightFish handler, the req object is named in the input parameter
        Ok(EightFishResponse::new(Status::Successful, req.method(), $results))
    };
}
