use serde::Serialize;

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
    result: Option<String>,
    custom_result: Option<String>,
    model_name: Option<String>,
    pair_list: Option<Vec<(String, String)>>,
}

impl EightFishResponse {
    pub fn new<T: EightFishModel + Serialize>(status: Status, result: Vec<T>) -> Self {
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

        EightFishResponse {
            status,
            result: Some(data),
            custom_result,
            model_name,
            pair_list: Some(pair_list),
        }
    }

    pub fn from_str(status: Status, custom_result: String) -> Self {
        EightFishResponse {
            status,
            result: None,
            custom_result: Some(custom_result),
            model_name: None,
            pair_list: None,
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
