use eightfish::EightFishModel;
use eightfish_derive::EightFishModel;
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
    assert_eq!("id, title, content", Foo::field_names());
}
#[test]
fn test_get_one_sql() {
    assert_eq!(
        "SELECT id, title, content FROM foo WHERE id = $1",
        Foo::build_get_one_sql()
    );
}

#[test]
fn test_get_all_sql() {
    assert_eq!(
        "SELECT id, title, content FROM foo",
        Foo::build_get_all_sql(None, None)
    );
    assert_eq!(
        "SELECT id, title, content FROM foo LIMIT 10 OFFSET 10",
        Foo::build_get_all_sql(Some(10), Some(10))
    );
}

#[test]
fn test_insert_sql() {
    assert_eq!(
        "INSERT INTO foo(id, title, content) VALUES ($1, $2, $3)",
        Foo::build_insert_sql()
    );
}

#[test]
fn test_update_sql() {
    assert_eq!(
        "UPDATE foo SET id = $2, title = $3, content = $4 WHERE id = $1",
        Foo::build_update_sql()
    );
}
#[test]
fn test_delete_sql() {
    assert_eq!("DELETE FROM foo WHERE id = $1", Foo::build_delete_sql());
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
    let params = f.build_insert_params();
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
    let params = f.build_insert_params();
    assert!(matches!(params[0], ParameterValue::Str("id")));
    assert!(matches!(params[2], ParameterValue::Boolean(true)));
    assert!(matches!(params[3], ParameterValue::Int64(100)));
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
    let params = f.build_update_params();
    assert!(matches!(params[0], ParameterValue::Str(id)));
    assert!(matches!(params[1], ParameterValue::Str(title)));
    assert!(matches!(params[2], ParameterValue::Str(content)));
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
    let (statement, params) = f.build_insert_sql_and_params();
    assert_eq!(
        "INSERT INTO foo(id, title, content) VALUES ($1, $2, $3)",
        statement
    );
    assert!(matches!(params[0], ParameterValue::Str(id)));
    assert!(matches!(params[1], ParameterValue::Str(title)));
    assert!(matches!(params[2], ParameterValue::Str(content)));
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
    let (statement, params) = f.build_update_sql_and_params();

    assert_eq!(
        "UPDATE foo SET id = $2, title = $3, content = $4 WHERE id = $1",
        statement
    );
    assert!(matches!(params[0], ParameterValue::Str(id)));
    assert!(matches!(params[1], ParameterValue::Str(title)));
    assert!(matches!(params[2], ParameterValue::Str(content)));
}

#[test]
fn test_build_get_one_sql_and_params() {
    let id = "id";
    let (statement, params) = Foo::build_get_one_sql_and_params(id);

    assert_eq!(
        "SELECT id, title, content FROM foo WHERE id = $1",
        statement
    );
    assert!(matches!(params[0], ParameterValue::Str(id)));
}

#[test]
fn test_build_delete_sql_and_params() {
    let id = "id";
    let (statement, params) = Foo::build_delete_sql_and_params(id);

    assert_eq!("DELETE FROM foo WHERE id = $1", statement);
    assert!(matches!(params[0], ParameterValue::Str(id)));
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
        "id = $2, title = $3, content = $4",
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
