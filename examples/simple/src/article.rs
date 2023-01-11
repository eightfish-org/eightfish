use eightfish::{Module, Request, Response, Result as EightFishResult, Router};


const REDIS_URL_ENV: &str = "REDIS_URL";
const DB_URL_ENV: &str = "DB_URL";

#[derive(Debug, Clone, Serialize, Deserialize, CalcHash)]
pub struct Article {
    id: String,
    title: String,
    content: String,
    authorname: String,
}

fn calc_hash<T: Serialize>(obj: &T) -> Result<String> {
    // I think we can use json_digest to do the deterministic hash calculating
    // https://docs.rs/json-digest/0.0.16/json_digest/
    let json_val= serde_json::to_value(obj).unwrap();
    let digest = json_digest::digest_data(&json_val).unwrap();

    Ok(digest)
}



pub struct ArticleModule;

impl ArticleModule {
    fn get_one(_req: &mut Request) -> EightFishResult<Response> {
        let pg_addr = std::env::var(DB_URL_ENV)?;
        let redis_addr = std::env::var(REDIS_URL_ENV)?;

        // first part, we need to parse the params data into a structure
        let mut params: HashMap<String, String> = HashMap::new();

        if data.is_some() {
            let _parse = form_urlencoded::parse(&data.as_ref().unwrap().as_bytes());
            // Iterate this _parse, push values into params
            for pair in _parse {
                let key = pair.0.to_string();
                let val = pair.1.to_string();
                params.insert(key, val);
            }
        }

        let article_id = params.get("id").unwrap();
        // construct a sql statement 
        let query_string = format!("select id, title, content, author from article where id='{article_id}'");
        let rowset = pg::query(&pg_addr, &query_string, &vec![]).unwrap();

        // convert the raw vec[u8] to every rust struct filed, and convert the whole into a
        // rust struct vec, later we may find a gerneral type converter way
        let mut results: Vec<Article> = vec![];
        for row in rowset.rows {
            let id = String::decode(&row[0])?;
            let title = String::decode(&row[1])?;
            let content = String::decode(&row[2])?;
            let authorname = String::decode(&row[3])?;

            let article = Article {
                id,
                title,
                content,
                authorname,
            };

            results.push(article);
        }

        let response = Response::new(
            Status::Successful, 
            "article:get_one".to_string(), 
            results);

        Ok(response)
    }

    fn new(req: &mut Request) -> EightFishResult<Response> {

        let pg_addr = std::env::var(DB_URL_ENV)?;
        let redis_addr = std::env::var(REDIS_URL_ENV)?;

        let mut params: HashMap<String, String> = HashMap::new();
        if data.is_some() {
            // first part, we need to parse the params data into a structure
            let _parse = form_urlencoded::parse(&data.as_ref().unwrap().as_bytes());

            // Iterate this _parse, push values into params
            for pair in _parse {
                let key = pair.0.to_string();
                let val = pair.1.to_string();
                params.insert(key, val);
            }
        }

        let title = params.get("title").unwrap();
        let content = params.get("content").unwrap();
        let authorname = params.get("authorname").unwrap();

        let id = Uuid::new_v4().simple().to_string(); // uuid

        // construct a struct
        let article = Article {
            id: id.clone(),
            title: title.clone(),
            content: content.clone(),
            authorname: authorname.clone(),
        };

        // construct a sql statement 
        let sql_string = format!("insert into article values ({}, {}, {}, {})", article.id, article.title, article.content, article.authorname);
        let _execute_results = pg::execute(&pg_addr, &sql_string, &vec![]);

        let mut results: Vec<Article> = vec![];
        results.push(article);

        let response = Response::new(
            Status::Successful, 
            "article:new".to_string(), 
            results);

        Ok(response)
    }

    fn delete(req: &mut Request) -> EightFishResult<Response> {
        let pg_addr = std::env::var(DB_URL_ENV)?;
        let redis_addr = std::env::var(REDIS_URL_ENV)?;

        let mut params: HashMap<String, String> = HashMap::new();
        if data.is_some() {
            // first part, we need to parse the params data into a structure
            let _parse = form_urlencoded::parse(&data.as_ref().unwrap().as_bytes());

            // Iterate this _parse, push values into params
            for pair in _parse {
                let key = pair.0.to_string();
                let val = pair.1.to_string();
                params.insert(key, val);
            }
        }

        let id = params.get("id").unwrap();

        // construct a sql statement 
        let sql_string = format!("delete article where id='{id}'");
        let _execute_results = pg::execute(&pg_addr, &sql_string, &vec![]);
        // TODO check the pg result

        let results: Vec<Article> = vec![];

        let response = Response::new(
            Status::Successful, 
            "article:delete:{id}".to_string(), 
            results);

        Ok(response)
    }
}

impl Module for EightFishModule {
    fn router(&self, router: &mut Router) -> EightFishResult<()> {
        router.get("/article/:id", Self::get_one);
        //router.get("/article/latest", Self::get_latest);
        router.post("/article/new", Self::new);
        //router.post("/article/update", Self::update);
        router.post("/article/delete/:id", Self::delete);

        Ok(())
    }
}

