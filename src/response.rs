pub enum Status {
    Successful,
    Failed,
}

pub struct EightResponse<T: IdHashPair + Serialize> {
    status: Status,
    info: String,
    results: Vec<T>
}

impl EightFishResponse<T> {
    pub fn new() -> EightFishResponse<T> {
        EightFishResponse {
            status: Status::Successful,
            info: String::new(),
            results: Vec::new(),
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

    /// get response info
    pub fn info(&self) -> &String {
        self.info
    }

    /// set response info
    pub fn set_info(&mut self, info: String) {
        self.info = info;
    }

    /// get response results
    pub fn results(&self) -> &<Vec<T>> {
        &self.results
    }

    /// set results
    pub fn set_results(&mut self, results: Vec<T>) {
        self.results = results;
    }

}
