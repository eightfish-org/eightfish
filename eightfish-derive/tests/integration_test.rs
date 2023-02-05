use eightfish::EightFishModel;
use eightfish_derive::EightFishModel;
use serde::{Deserialize, Serialize};
use spin_sdk::pg::ParameterValue;
#[derive(Default, EightFishModel, PartialEq, Debug, Serialize, Deserialize)]
struct Foo {
    id: String,
    title: String,
    content: String,
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
        Foo::get_one_sql()
    );
}

#[test]
fn test_get_all_sql() {
    assert_eq!("SELECT id, title, content FROM foo", Foo::get_all_sql());
}

#[test]
fn test_insert_sql() {
    assert_eq!(
        "INSERT INTO foo(id, title, content) VALUES ($1, $2, $3)",
        Foo::insert_sql()
    );
}

#[test]
fn test_update_sql() {
    assert_eq!(
        "UPDATE foo SET id = $2, title = $3, content = $4 WHERE id = $1",
        Foo::update_sql()
    );
}
#[test]
fn test_delete_sql() {
    assert_eq!("DELETE FROM foo WHERE id = $1", Foo::delete_sql());
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
    let params = f.build_insert_param();
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
fn test_build_update_param() {
    let id = "id".to_string();
    let title = "my blog".to_string();
    let content = "blog content".to_string();
    let f = Foo {
        id: id.clone(),
        title: title.clone(),
        content: content.clone(),
    };
    let params = f.build_update_param();
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
fn test_build_sql_insert() {
    let id = "id".to_string();
    let title = "my blog".to_string();
    let content = "blog content".to_string();
    let f = Foo {
        id: id.clone(),
        title: title.clone(),
        content: content.clone(),
    };
    let (statement, params) = f.build_sql_insert();
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
    assert_eq!(
        "INSERT INTO foo(id, title, content) VALUES ($1, $2, $3)",
        statement
    );
    assert_eq!(vec!["id", "my blog", "blog content"], params_str);
}

#[test]
fn test_build_sql_update() {
    let id = "id".to_string();
    let title = "my blog".to_string();
    let content = "blog content".to_string();
    let f = Foo {
        id: id.clone(),
        title: title.clone(),
        content: content.clone(),
    };
    let (statement, params) = f.build_sql_update();
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

    assert_eq!(
        "UPDATE foo SET id = $2, title = $3, content = $4 WHERE id = $1",
        statement
    );
    assert_eq!(vec!["id", "my blog", "blog content"], params_str);
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
fn test_convert_struct_to_row() {
    let id = "id".to_string();
    let title = "my blog".to_string();
    let content = "blog content".to_string();
    let f = Foo {
        id: id.clone(),
        title: title.clone(),
        content: content.clone(),
    };
    let expected = vec!["hash".to_string(), id, title, content];
    assert_eq!(expected, f.to_row("hash".to_string()));
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
    let row = vec!["hash".to_string(), id, title, content];
    assert_eq!(expected, Foo::from_row(row));
}
