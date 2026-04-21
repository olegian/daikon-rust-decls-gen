struct MyStruct(u32);
impl MyStruct {
    fn bar(self) {
        println!("This function is inside an impl");
    }
}

fn main() {
    println!("Hello, world!");
}

fn foo() {
    println!("This is a different function.");
}