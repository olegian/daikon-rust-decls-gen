struct Root(Child1, u32);
struct Child1(Child2, u32);
struct Child2(Child3, u32);
struct Child3(Child4, u32);
struct Child4(Child5, u32);
struct Child5(Child6, u32);
struct Child6(bool, u32);

fn main() {
    println!("Hello, world!");
}

fn root(x: Root) {
    println!("foo");
}

fn leaf(x: Child6) {
    println!("bar")
}
