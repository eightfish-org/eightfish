use eightfish::{EightFishModel, Info, Module, Request, Response, Result, Router, Status};
use eightfish_derive::EightFishModel;
use serde::{Deserialize, Serialize};
use spin_sdk::pg::{self, Decode, ParameterValue};
use uuid::Uuid;

const REDIS_URL_ENV: &str = "REDIS_URL";
const DB_URL_ENV: &str = "DB_URL";

#[derive(Debug, Clone, Serialize, Deserialize, EightFishModel, Default)]
pub struct Article {
    id: String,
    title: String,
    content: String,
    authorname: String,
}

pub struct ArticleModule;

impl ArticleModule {
    fn get_one(req: &mut Request) -> Result<Response> {
        let pg_addr = std::env::var(DB_URL_ENV).unwrap();

        let params = req.parse_urlencoded();

        let article_id = params.get("id").unwrap();

        // construct a sql statement
        let query_string = Article::get_one_sql();
        let params = vec![ParameterValue::Str(article_id.as_str())];
        let rowset = pg::query(&pg_addr, &query_string, &params).unwrap();

        // convert the raw vec[u8] to every rust struct filed, and convert the whole into a
        // rust struct vec, later we may find a gerneral type converter way
        let mut results: Vec<Article> = vec![];
        for row in rowset.rows {
            let row_string = row
                .iter()
                .map(|c| String::decode(c).unwrap())
                .collect::<Vec<String>>();
            let article = Article::from_row(row_string);

            results.push(article);
        }

        let info = Info {
            model_name: "article".to_string(),
            action: "get_one".to_string(),
            target: article_id.clone(),
            extra: "".to_string(),
        };

        let response = Response::new(Status::Successful, info, results);

        Ok(response)
    }

    fn new(req: &mut Request) -> Result<Response> {
        let pg_addr = std::env::var(DB_URL_ENV).unwrap();

        let params = req.parse_urlencoded();

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
        let sql_string = Article::insert_sql();
        let sql_params = article.build_insert_param();
        let _execute_results = pg::execute(&pg_addr, &sql_string, &sql_params);

        let mut results: Vec<Article> = vec![];
        results.push(article);

        let info = Info {
            model_name: "article".to_string(),
            action: "new".to_string(),
            target: id.clone(),
            extra: "".to_string(),
        };

        let response = Response::new(Status::Successful, info, results);

        Ok(response)
    }

    fn update(req: &mut Request) -> Result<Response> {
        let pg_addr = std::env::var(DB_URL_ENV).unwrap();

        let params = req.parse_urlencoded();

        let id = params.get("id").unwrap();
        let title = params.get("title").unwrap();
        let content = params.get("content").unwrap();
        let authorname = params.get("authorname").unwrap();

        // construct a struct
        let article = Article {
            id: id.clone(),
            title: title.clone(),
            content: content.clone(),
            authorname: authorname.clone(),
        };

        // construct a sql statement
        let sql_string = Article::update_sql();
        let sql_params = article.build_update_param();
        let _execute_results = pg::execute(&pg_addr, &sql_string, &sql_params);

        let mut results: Vec<Article> = vec![];
        results.push(article);

        let info = Info {
            model_name: "article".to_string(),
            action: "update".to_string(),
            target: id.clone(),
            extra: "".to_string(),
        };

        let response = Response::new(Status::Successful, info, results);

        Ok(response)
    }

    fn delete(req: &mut Request) -> Result<Response> {
        let pg_addr = std::env::var(DB_URL_ENV).unwrap();

        let params = req.parse_urlencoded();

        let id = params.get("id").unwrap();

        // construct a sql statement
        let sql_string = Article::delete_sql();
        let params = vec![ParameterValue::Str(id.as_str())];
        let _execute_results = pg::execute(&pg_addr, &sql_string, &params);
        // TODO check the pg result

        let results: Vec<Article> = vec![];

        let info = Info {
            model_name: "article".to_string(),
            action: "delete".to_string(),
            target: id.clone(),
            extra: "".to_string(),
        };

        let response = Response::new(Status::Successful, info, results);

        Ok(response)
    }
}

impl Module for ArticleModule {
    fn router(&self, router: &mut Router) -> Result<()> {
        router.get("/article/:id", Self::get_one);
        //router.get("/article/latest", Self::get_latest);
        router.post("/article/new", Self::new);
        router.post("/article/update", Self::update);
        router.post("/article/delete/:id", Self::delete);

        Ok(())
    }
}
