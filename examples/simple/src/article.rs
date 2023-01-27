use eightfish::{Module, Request, Response, Result, Router, Status, Info, EightFishModel};
use serde::{Serialize, Deserialize};
use spin_sdk::{
    pg::{self, Decode},
};
use uuid::Uuid;

const REDIS_URL_ENV: &str = "REDIS_URL";
const DB_URL_ENV: &str = "DB_URL";

//#[derive(Debug, Clone, Serialize, Deserialize, EightFishModel)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Article {
    id: String,
    title: String,
    content: String,
    authorname: String,
}

impl EightFishModel for Article {
    fn id(&self) -> String {
        self.id.clone()
    }

    fn calc_hash(&self) -> String {
        let json_val= serde_json::to_value(self).unwrap();
        let digest = json_digest::digest_data(&json_val).unwrap();

        digest
    }
}

pub struct ArticleModule;

impl ArticleModule {
    fn get_one(req: &mut Request) -> Result<Response> {
        let pg_addr = std::env::var(DB_URL_ENV).unwrap();

        let params = req.parse_urlencoded().unwrap();

        let article_id = params.get("id")?;
        // construct a sql statement 
        let query_string = format!("select id, title, content, author from article where id='{article_id}'");
        let rowset = pg::query(&pg_addr, &query_string, &vec![]).unwrap();

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

        let params = req.parse_urlencoded().unwrap();

        let title = params.get("title")?;
        let content = params.get("content")?;
        let authorname = params.get("authorname")?;

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

        let params = req.parse_urlencoded().unwrap();

        let id = params.get("id")?;
        let title = params.get("title")?;
        let content = params.get("content")?;
        let authorname = params.get("authorname")?;

        // construct a struct
        let article = Article {
            id: id.clone(),
            title: title.clone(),
            content: content.clone(),
            authorname: authorname.clone(),
        };

        // construct a sql statement 
        let sql_string = format!("update article set id='{}', title='{}', content='{}', authorname='{}' where id='{id}'", article.id, article.title, article.content, article.authorname);
        let _execute_results = pg::execute(&pg_addr, &sql_string, &vec![]);

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

        let params = req.parse_urlencoded().unwrap();

        let id = params.get("id")?;

        // construct a sql statement 
        let sql_string = format!("delete article where id='{id}'");
        let _execute_results = pg::execute(&pg_addr, &sql_string, &vec![]);
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

