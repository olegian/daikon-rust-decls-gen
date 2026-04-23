mod dep;

struct MyStruct(u32);
impl MyStruct {
    fn bar(self) {
        println!("This function is inside an impl");
    }
}

enum MyEnum {
    V1(u32, f64),
    V2 {
        a: u64,
        b: bool
    },
    V3
}

fn main() {
    println!("Hello, world!");
}

fn foo(p1: MyEnum, p2: MyStruct, p3: Vec<u32>, p4: dep::DepStruct<i32>) {
    println!("This is a different function.");
}

fn baz(p1: [[u32; 3]; 3]) -> MyStruct {
    return MyStruct(0);
}

fn early(x: i32) -> i32 {
    if x < 0 {
        return 0;
    }
    x + 1
}