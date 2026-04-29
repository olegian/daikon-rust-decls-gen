mod dep;

#[derive(Debug)]
struct MyStruct<T>(u32, T);
impl<T> MyStruct<T> {
    fn bar(&self, param: T) {
        println!("This function is inside an impl");
    }

    fn accepts_self(self, param: u32) -> Self {
        self
    }
}

enum MyEnum {
    V1(u32, f64),
    V2 { a: u64, b: bool },
    V3,
}

fn main() {
    quux((10, false, 10));
    quux((10, false, "string"));
    println!("Hello, world!");
    let a = baz([[10; 3]; 3]);
    println!("{:?}", a);
}

fn foo(p1: &mut MyEnum, p2: MyStruct<&str>, p3: Vec<String>, p4: dep::DepStruct<i32>) {
    p2.bar("hello");
    println!("This is a different function.");
}

fn baz(p1: [[u32; 3]; 3]) -> MyStruct<i32> {
    let res = MyStruct(0, 10);
    res.bar(20);
    res
}

fn quux<T>(p1: (u32, bool, T)) -> T {
    println!("function accepts tuple!");
    p1.2
}

fn early(x: i32, box_p: Box<u32>) -> i32 {
    if x < 0 {
        return 0;
    }
    x + 1
}
