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

fn bar(p1: [[u32; 3]; 3]) {}