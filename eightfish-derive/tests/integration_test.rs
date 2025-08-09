use eightfish_derive::{dtocore, EightFishDTO, EightFishModel};
use eightfish_sdk::EightFishModel;
use serde::{Deserialize, Serialize};
use spin_sdk::pg::{DbValue, Decode, ParameterValue};

#[derive(Default, EightFishModel, PartialEq, Debug, Serialize, Deserialize)]
struct Foo {
    id: String,
    title: String,
    content: String,
}

#[derive(Default, EightFishModel, PartialEq, Debug, Serialize, Deserialize)]
struct FooMix {
    id: String,
    title: String,
    synced: bool,
    like: i64,
}

#[test]
fn test_model_names() {
    assert_eq!("foo", Foo::model_name());
}

#[test]
fn test_struct_names() {
    let vec = vec![
        "foo.id".to_string(),
        "foo.title".to_string(),
        "foo.content".to_string(),
    ];
    assert_eq!(vec, Foo::fields());
}
#[test]
fn test_get_one_sql() {
    assert_eq!(
        "SELECT foo.id, foo.title, foo.content FROM foo WHERE id = $1;",
        Foo::sql_get_by_id()
    );
}

#[test]
fn test_insert_sql() {
    assert_eq!(
        "INSERT INTO foo (foo.id, foo.title, foo.content) VALUES ($1, $2, $3) RETURNING *;",
        Foo::sql_insert()
    );
}

#[test]
fn test_update_sql() {
    assert_eq!(
        "UPDATE foo SET id = $1, title = $2, content = $3 WHERE id = $1 RETURNING *;",
        Foo::sql_update()
    );
}
#[test]
fn test_delete_sql() {
    assert_eq!(
        "DELETE FROM foo WHERE id = $1 RETURNING *;",
        Foo::sql_delete()
    );
}

#[test]
fn test_build_insert_param() {
    let id = "id".to_string();
    let title = "my blog".to_string();
    let content = "blog content".to_string();
    let f = Foo {
        id: id.clone(),
        title: title.clone(),
        content: content.clone(),
    };
    let params = f.params_insert();
    let params_str = params
        .iter()
        .map(|p| {
            if let ParameterValue::Str(s) = p {
                format!("{}", s)
            } else {
                format!("{}", "None")
            }
        })
        .collect::<Vec<String>>();
    assert_eq!(vec!["id", "my blog", "blog content"], params_str);
}

#[test]
fn test_build_insert_param_mix_type() {
    let id = "id".to_string();
    let title = "my blog".to_string();
    let f = FooMix {
        id: id.clone(),
        title: title.clone(),
        synced: true,
        like: 100,
    };
    let params = f.params_insert();
    assert!(matches!(&params[0], ParameterValue::Str(_id)));
    assert!(matches!(&params[2], ParameterValue::Boolean(true)));
    assert!(matches!(&params[3], ParameterValue::Int64(100)));
}

#[test]
fn test_build_update_param() {
    let id = "id";
    let title = "my blog";
    let content = "blog content";
    let f = Foo {
        id: id.to_string(),
        title: title.to_string(),
        content: content.to_string(),
    };
    let params = f.params_update();
    assert!(matches!(&params[0], ParameterValue::Str(_id)));
    assert!(matches!(&params[1], ParameterValue::Str(_title)));
    assert!(matches!(&params[2], ParameterValue::Str(_content)));
}
#[test]
fn test_build_insert_sql_and_params() {
    let id = "id";
    let title = "my blog";
    let content = "blog content";
    let f = Foo {
        id: id.to_string(),
        title: title.to_string(),
        content: content.to_string(),
    };
    let (statement, params) = f.build_insert();
    assert_eq!(
        "INSERT INTO foo (foo.id, foo.title, foo.content) VALUES ($1, $2, $3) RETURNING *;",
        statement
    );
    assert!(matches!(&params[0], ParameterValue::Str(_id)));
    assert!(matches!(&params[1], ParameterValue::Str(_title)));
    assert!(matches!(&params[2], ParameterValue::Str(_content)));
}

#[test]
fn test_build_update_sql_and_params() {
    let id = "id".to_string();
    let title = "my blog".to_string();
    let content = "blog content".to_string();
    let f = Foo {
        id: id.clone(),
        title: title.clone(),
        content: content.clone(),
    };
    let (statement, params) = f.build_update();

    assert_eq!(
        "UPDATE foo SET id = $1, title = $2, content = $3 WHERE id = $1 RETURNING *;",
        statement
    );
    assert!(matches!(&params[0], ParameterValue::Str(_id)));
    assert!(matches!(&params[1], ParameterValue::Str(_title)));
    assert!(matches!(&params[2], ParameterValue::Str(_content)));
}

#[test]
fn test_build_get_one_sql_and_params() {
    let id = "id";
    let (statement, params) = Foo::build_get_by_id(id);

    assert_eq!(
        "SELECT foo.id, foo.title, foo.content FROM foo WHERE id = $1;",
        statement
    );
    assert!(matches!(&params[0], ParameterValue::Str(_id)));
}

#[test]
fn test_build_delete_sql_and_params() {
    let id = "id";
    let title = "my blog";
    let content = "blog content";
    let f = Foo {
        id: id.to_string(),
        title: title.to_string(),
        content: content.to_string(),
    };
    let (statement, params) = f.build_delete();

    assert_eq!("DELETE FROM foo WHERE id = $1 RETURNING *;", statement);
    assert!(matches!(&params[0], ParameterValue::Str(_id)));
}

#[test]
fn test_get_id() {
    let id = "id".to_string();
    let title = "my blog".to_string();
    let content = "blog content".to_string();
    let f = Foo {
        id: id.clone(),
        title: title.clone(),
        content: content.clone(),
    };
    assert_eq!("id".to_string(), f.id());
}

#[test]
fn test_calc_hash() {
    let id = "id".to_string();
    let title = "my blog".to_string();
    let content = "blog content".to_string();
    let f = Foo {
        id: id.clone(),
        title: title.clone(),
        content: content.clone(),
    };
    assert_eq!(
        "cjuOl837GfI9V1rYe2iJZw9a5cLae_QiWWLoYL7-IXgLNA".to_string(),
        f.calc_hash()
    );
}

#[test]
fn test_struct_names_placeholder() {
    assert_eq!("$1, $2, $3", Foo::row_placeholders());
}

#[test]
fn test_struct_names_update_placeholder() {
    assert_eq!(
        "id = $1, title = $2, content = $3",
        Foo::update_placeholders()
    );
}

#[test]
fn test_build_struct_from_row() {
    let id = "id".to_string();
    let title = "my blog".to_string();
    let content = "blog content".to_string();

    let expected = Foo {
        id: id.clone(),
        title: title.clone(),
        content: content.clone(),
    };
    //println!("{}", Foo::field_names());
    let row = vec![
        DbValue::Str(id.clone()),
        DbValue::Str(title.clone()),
        DbValue::Str(content.clone()),
    ];
    assert_eq!(expected, Foo::from_row(row));
}

#[derive(Default, EightFishModel, PartialEq, Debug, Serialize, Deserialize)]
struct StructInner {
    id: String, // it's must
    a: String,
    b: String,
}

#[derive(EightFishDTO, Debug, Serialize, Deserialize)]
struct StructDTO {
    #[dtocore]
    inner: StructInner,
    other: String,
}

#[test]
fn test_ef_dto() {
    let row = vec![
        DbValue::Str("xxuuid".to_string()),
        DbValue::Str("1".to_string()),
        DbValue::Str("test".to_string()),
        DbValue::Str("extra".to_string()),
    ];
    let dto = StructDTO::from_row(row);
    println!("{:?}", dto);

    assert_eq!(dto.inner.id, "xxuuid");
    assert_eq!(dto.inner.a, "1");
    assert_eq!(dto.inner.b, "test");
    assert_eq!(dto.other, "extra");
}

#[derive(Default, EightFishModel, PartialEq, Debug, Serialize, Deserialize)]
struct Inner0 {
    id: String, // it's must
    a: i32,
    b: String,
}

#[derive(Default, EightFishModel, PartialEq, Debug, Serialize, Deserialize)]
struct FooWithOptions {
    id: String,
    title: Option<String>,
    count: Option<i64>,
    active: Option<bool>,
}

#[derive(EightFishDTO, Debug, Serialize, Deserialize)]
struct DTO00 {
    #[dtocore]
    inner: Inner0,
    other: String,
}

#[test]
fn test_ef_dto_2() {
    let row = vec![
        DbValue::Str("xxuuid".to_string()),
        DbValue::Int32(1),
        DbValue::Str("test".to_string()),
        DbValue::Str("extra".to_string()),
    ];
    let dto = DTO00::from_row(row);
    println!("{:?}", dto);

    assert_eq!(dto.inner.id, "xxuuid");
    assert_eq!(dto.inner.a, 1);
    assert_eq!(dto.inner.b, "test");
    assert_eq!(dto.other, "extra");

    assert_eq!(dto.id(), "xxuuid");
    assert_eq!(dto.inner.id(), "xxuuid");
    assert_eq!(dto.calc_hash(), dto.inner.calc_hash());
}

#[test]
fn test_option_fields_with_values() {
    let f = FooWithOptions {
        id: "test_id".to_string(),
        title: Some("test_title".to_string()),
        count: Some(42),
        active: Some(true),
    };

    let params = f.params_insert();
    assert_eq!(params.len(), 4);

    assert!(matches!(&params[0], ParameterValue::Str(s) if s == "test_id"));
    assert!(matches!(&params[1], ParameterValue::Str(s) if s == "test_title"));
    assert!(matches!(&params[2], ParameterValue::Int64(42)));
    assert!(matches!(&params[3], ParameterValue::Boolean(true)));
}

#[test]
fn test_option_fields_with_none_values() {
    let f = FooWithOptions {
        id: "test_id".to_string(),
        title: None,
        count: None,
        active: None,
    };

    let params = f.params_insert();
    assert_eq!(params.len(), 4);

    assert!(matches!(&params[0], ParameterValue::Str(s) if s == "test_id"));
    assert!(matches!(&params[1], ParameterValue::DbNull));
    assert!(matches!(&params[2], ParameterValue::DbNull));
    assert!(matches!(&params[3], ParameterValue::DbNull));
}

#[test]
fn test_option_fields_mixed_values() {
    let f = FooWithOptions {
        id: "test_id".to_string(),
        title: Some("test_title".to_string()),
        count: None,
        active: Some(false),
    };

    let params = f.params_insert();
    assert_eq!(params.len(), 4);

    assert!(matches!(&params[0], ParameterValue::Str(s) if s == "test_id"));
    assert!(matches!(&params[1], ParameterValue::Str(s) if s == "test_title"));
    assert!(matches!(&params[2], ParameterValue::DbNull));
    assert!(matches!(&params[3], ParameterValue::Boolean(false)));
}
