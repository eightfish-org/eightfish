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
        let fields = Article::field_names();
        let table = Article::model_name();
        let params = vec![ParameterValue::Str(article_id.as_str())];
        // construct a sql statement
        let query_string = format!("SELECT {fields} FROM {table} WHERE id = $1");
        let rowset = pg::query(&pg_addr, &query_string, &params).unwrap();

        // convert the raw vec[u8] to every rust struct filed, and convert the whole into a
        // rust struct vec, later we may find a gerneral type converter way
        let mut results: Vec<Article> = vec![];
        for row in rowset.rows {
            let id = String::decode(&row[0]).unwrap();
            let title = String::decode(&row[1]).unwrap();
            let content = String::decode(&row[2]).unwrap();
            let authorname = String::decode(&row[3]).unwrap();

            let article = Article {
                id,
                title,
                content,
                authorname,
            };
            //let article = Article::from_row(&row);

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
        let fields = Article::field_names();
        let fields_placeholder = Article::row_placeholders();
        let table = Article::model_name();
        let params = vec![
            ParameterValue::Str(article.id.as_str()),
            ParameterValue::Str(article.title.as_str()),
            ParameterValue::Str(article.content.as_str()),
            ParameterValue::Str(article.authorname.as_str()),
        ];
        // construct a sql statement
        let sql_string = format!("INSERT INTO {table}({fields}) VALUES ({fields_placeholder})");
        let _execute_results = pg::execute(&pg_addr, &sql_string, &params);

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
        let table = Article::model_name();
        let update_filed_placeholder = Article::update_placeholders();
        let params = vec![
            ParameterValue::Str(article.title.as_str()),
            ParameterValue::Str(article.content.as_str()),
            ParameterValue::Str(article.authorname.as_str()),
            ParameterValue::Str(article.id.as_str()),
        ];
        // construct a sql statement
        let sql_string = format!("UPDATE {table} SET {update_filed_placeholder} WHERE id = $1");
        let _execute_results = pg::execute(&pg_addr, &sql_string, &params);

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
        let params = vec![ParameterValue::Str(id.as_str())];
        let table = Article::model_name();
        // construct a sql statement
        let sql_string = format!("DELETE {table} WHERE id = $1");
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
