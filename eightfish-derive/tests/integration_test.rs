use eightfish_derive::EightFishModel;
#[derive(Default, EightFishModel, PartialEq, Debug)]
struct Foo {
    id: String,
    title: String,
    content: String,
}

#[test]
fn test_struct_names() {
    assert_eq!("id, title, content", Foo::field_names());
}

#[test]
fn test_struct_names_placeholder() {
    assert_eq!("$1, $2, $3", Foo::row_placeholders());
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
