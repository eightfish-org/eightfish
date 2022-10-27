use eightfish_derive::MyTrait;

/*
trait MyTrait {
    fn answer() -> i32 {
        42
    }
}
*/

#[derive(MyTrait)]
struct Foo {
    id: String,
    title: String,
    content: String,
    timestamp: u64,
}

fn main() {
    Foo::describe();
}

#[test]
fn default() {
    //assert_eq!(Foo::answer(), 42);
    Foo::describe();
}
