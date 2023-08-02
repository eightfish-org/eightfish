# CRUD



## Create

You can use the method of `instance.build_insert()` to get an insert SQL statement from a model. For example:

```
let article = Article {
    id,
    title,
    content,
    authorname,
};

let (sql_statement, sql_params) = article.build_insert();
_ = pg::execute(&pg_addr, &sql_statement, &sql_params)?;
```

Please refer [here](https://github.com/eightfish-org/ef_example_simple_standalone/blob/master/src/article.rs#L76) to checkout the context.

## Update

You can use the method of `instance.build_update()` to get an update SQL statement from a model. For example:

```
let article = Article {
    id,
    title,
    content,
    authorname,
    ..old_article
};

let (sql, sql_params) = article.build_update();
_ = pg::execute(&pg_addr, &sql, &sql_params)?;
```

Please refer [here](https://github.com/eightfish-org/ef_example_simple_standalone/blob/master/src/article.rs#L123) to checkout the context.

## Delete

You can use the method of `model_name::build_delete(id)` to get a delete SQL statement from a model. For example:

```
let id = params.get("id").ok_or(anyhow!("id error"))?;

let (sql, sql_params) = Article::build_delete(id);
_ = pg::execute(&pg_addr, &sql, &sql_params)?;
```

Please refer [here](https://github.com/eightfish-org/ef_example_simple_standalone/blob/master/src/article.rs#L149) to checkout the context.

## Get by Id


You can use the method of `model_name::build_get_by_id()(id)` to get a simple query SQL statement from a model. For example:

```
let article_id = params.get("id").ok_or(anyhow!("id error"))?;

let (sql, sql_params) = Article::build_get_by_id(article_id);
let rowset = pg::query(&pg_addr, &sql, &sql_params)?;
```

Please refer [here](https://github.com/eightfish-org/ef_example_simple_standalone/blob/master/src/article.rs#L149) to checkout the context.

## Convert to a Rust type

You can use the method of `model_name::from_row(row)` to convert a row data from db to a specific Rust type (Model) instance. For example:

```
let article = Article::from_row(row);
```

Please refer [here](https://github.com/eightfish-org/ef_example_simple_standalone/blob/master/src/article.rs#L33) to checkout the context.

