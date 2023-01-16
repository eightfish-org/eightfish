use eightfish_derive::EightFishHelper;

#[derive(EightFishHelper, Debug, Default)]
struct Foo {
    id: String,
    title: String,
    content: String,
//    timestamp: u64,
}

fn main() {
    println!("{}", Foo::field_names());
    println!("{}", Foo::row_placeholders());

    let foo = Foo {
        id: "aaa".to_string(),
        title: "bbb".to_string(),
        content: "ccc".to_string(),
//        timestamp: 10000
    };

    let f2 = foo.to_vec();
    println!("{:?}", f2);

    let avec = vec!["ahash".to_string(), "aa".to_string(),"bb".to_string(), "cc".to_string()];
    let obj = Foo::from_row(avec);
    println!("{:?}", obj);
}

#[test]
fn default() {
    //assert_eq!(Foo::answer(), 42);
    Foo::describe();
}
