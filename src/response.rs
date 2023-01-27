use serde::{Serialize, Deserialize};
use anyhow::Result;

/// Response status
#[derive(Clone)]
pub enum Status {
    Successful,
    Failed,
}

/// Response info
#[derive(Default)]
pub struct Info {
    pub model_name: String,
    pub action: String,
    pub target: String,
    pub extra: String,
}

// code put here temporarily, this func would be put into the eightfish-derive crate to 
// implemented on struct
fn calc_hash<T: Serialize>(obj: &T) -> Result<String> {
    // I think we can use json_digest to do the deterministic hash calculating
    // https://docs.rs/json-digest/0.0.16/json_digest/
    let json_val= serde_json::to_value(obj).unwrap();
    let digest = json_digest::digest_data(&json_val).unwrap();

    Ok(digest)
}

pub trait EightFishModel: Serialize {
    fn id(&self) -> String;
    fn calc_hash(&self) -> String;
}


pub struct EightFishResponse {
    status: Status,
    info: Info,
    pair_list: Option<Vec<(String, String)>>,
    results: Option<String>,
}

fn do_serialization<T: Serialize>(results: Vec<T>) -> String {
    serde_json::to_string(&results).unwrap()
}


impl EightFishResponse {
    pub fn new<T: Serialize + EightFishModel>(status: Status, info: Info, aresults: Vec<T>) -> EightFishResponse {
       
        let mut pair_list;
        let mut results;

        if aresults.is_empty() {
            pair_list = None;
            results = None;
        } else {
            let a_pair_list = aresults.iter().map(|obj| (obj.id(), obj.calc_hash())).collect();
            pair_list = Some(a_pair_list);
            let output = do_serialization(aresults);
            results = Some(output);
        }

        EightFishResponse {
            status,
            info,
            pair_list,
            results,
        }
    }

    /// get response status
    pub fn status(&self) -> Status {
        self.status.clone()
    }

    /// set response status
    pub fn set_status(&mut self, status: Status) {
        self.status = status;
    }

    /// get response info
    pub fn info(&self) -> &Info {
        &self.info
    }

    /// set response info
    pub fn set_info(&mut self, info: Info) {
        self.info = info;
    }

    /// get response pair_list
    pub fn pair_list(&self) -> &Option<Vec<(String, String)>> {
        &self.pair_list
    }

    /// get response results
    pub fn results(&self) -> &Option<String> {
        &self.results
    }

    /// set results
    pub fn set_results(&mut self, results: Option<String>) {
        self.results = results;
    }

}
