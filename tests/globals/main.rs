struct MyStruct {
    a: u32,
    b: bool,
}

const fn create_struct() -> MyStruct {
    MyStruct {
        a: 0,
        b: false,
    }
}

const fn square(x: u32) -> u32 {
    x * x
}

const INT_CONST: u32 = 10 + 10;
const STR_CONST: &'static str = "this string is a const value";
const STRUCT_CONST: MyStruct = create_struct();
const ARRAY_CONST: [isize; 3] = [100; 3];
const FN_INITED_CONST: [u32; 3] = [square(1), square(2), square(3)];
const INT_CONST_FN: u32 = square(10);
const ARRAY_CONST_BLOCK: &[String; 3] = &[const { String::new() }; 3];

static STATIC_INT: i32 = 10;
static mut STATIC_MUT_INT: u32 = 20;
static mut STATIC_MUT_ARRAY: &'static [String; 3] = &[const { String::new() }; 3];

// Evil because this will evaluate to a value that might need
// to be weirdly represented in the decl file, specifically the 
// const tag that i aim to add to all `const` vars.
const EVIL_NESTED_ARRAY_CONST: [[[u32; 3]; 3]; 3] = [[[10; 3]; 3]; 3];

fn main() {
    println!("Hello, world!");
}

fn foo(x: i32) -> i32 {
    if x < 0 {
        return 0;
    }
    x + 1
}

fn bar(y: String) {
    println!("bar func")
}
