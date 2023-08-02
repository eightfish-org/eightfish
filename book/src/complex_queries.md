# Complex Queries

EightFish itself only supplies a set of basic sql builder helper function, if you need to construct complex sql statements, especially for query cases, you can use other sql builder crates to do it.

Ordinarily, we recommand to use crate `sql_builder` to do it.

Here are some examples:

nake sql.

```
  let sql = SqlBuilder::select_from(&GutpPost::model_name())
      .fields(&GutpPost::fields())
      .order_desc("created_time")
      .limit(limit)
      .offset(offset)
      .sql()?;
  let rowset = pg::query(&pg_addr, &sql, &[])?;
```

sql with parameters.

```
  let sql = SqlBuilder::select_from(&GutpPost::model_name())
      .fields(&GutpPost::fields())
      .and_where_eq("subspace_id", "$1")
      .order_desc("created_time")
      .limit(limit)
      .offset(offset)
      .sql()?;
  let sql_param = ParameterValue::Str(subspace_id);
  let rowset = pg::query(&pg_addr, &sql, &[sql_param])?;
```

With `sql_builder::SqlBuilder` you can cook any deep of complicated sql sentence.

