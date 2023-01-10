fn handle_query(reqid: &str, path: &str, data: &Option<String>) -> Result<String> {
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

    match path {
        "/article" => {
            // ----- biz logic part -----
            // get the view of one article, the parameter is in 'data', in the format of url
            // encoded
            let article_id = params.get("id").unwrap();
            // construct a sql statement 
            let query_string = format!("select hash, id, title, content, author from article where id='{article_id}'");
            let rowset = pg::query(&pg_addr, &query_string, &vec![]).unwrap();

            // convert the raw vec[u8] to every rust struct filed, and convert the whole into a
            // rust struct vec, later we may find a gerneral type converter way
            let mut results: Vec<ArticleHash> = vec![];
            for row in rowset.rows {
                let id = as_owned_string(&row[1])?;
                let title = as_owned_string(&row[2])?;
                let content = as_owned_string(&row[3])?;
                let authorname = as_owned_string(&row[4])?;
                let hash = as_owned_string(&row[0])?;

                let article = Article {
                    id,
                    title,
                    content,
                    authorname,
                };

                // MUST check the article obj and the hash value equlity get from db
                let checked_hash = calc_hash(&article).unwrap();
                if checked_hash != hash {
                     return Err(anyhow!("Hash mismatching.".to_string()))
                }

                let article_hash = ArticleHash {
                    article,
                    hash,
                };

                println!("article_hash: {:#?}", article_hash);
                results.push(article_hash);

                tail_query_process(&redis_addr, reqid, "Article", &results);

            }
        }
        "/article_info" => {

        }
        &_ => {
            todo!()
        }   
    }

    Ok("ok".to_string())
}

fn handle_event(reqid: &str, path: &str, data: &Option<String>) -> Result<String> {
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

    match path {
        "/article/new" => {
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

            // should ensure the serialization way is determined.
            // and the field hash won't participate the serialization
            let hash = calc_hash(&article).unwrap();

            // construct a sql statement 
            let sql_string = format!("insert into article values ({}, {}, {}, {}, {})", &hash, article.id, article.title, article.content, article.authorname);
            let _execute_results = pg::execute(&pg_addr, &sql_string, &vec![]);

            tail_post_process(&redis_addr, reqid, "Article", &id, &hash);
        }
        "/article/delete" => {
            let id = params.get("id").unwrap();

            // construct a sql statement 
            let sql_string = format!("delete article where id='{id}'");
            let _execute_results = pg::execute(&pg_addr, &sql_string, &vec![]);

            tail_post_process(&redis_addr, reqid, "Article", &id, "");
        }
        &_ => {
            todo!()
        }   

    }

    Ok("ok".to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let result = add(2, 2);
        assert_eq!(result, 4);
    }
}
