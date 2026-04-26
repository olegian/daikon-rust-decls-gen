struct MyStruct {
    a: u32,
    b: bool,
}

enum MyEnum {
    V1(u32),
    V2(f64),
}

const fn create_struct() -> MyStruct {
    MyStruct { a: 0, b: false }
}

const fn create_enum() -> MyEnum {
    MyEnum::V1(10)
}
 
const fn square(x: u32) -> u32 {
    x * x
}

const INT_CONST: u32 = 10 + 10;
const STR_CONST: &'static str = "this string is a const value";
const STRUCT_CONST: MyStruct = create_struct();
const ENUM_CONST: MyEnum = create_enum();
const ARRAY_CONST: [isize; 3] = [100; 3];
const FN_INITED_CONST: &[u32] = &[square(1), square(2), square(3)];
const INT_CONST_FN: u32 = square(10);
const ARRAY_CONST_BLOCK_STRING: &[String; 3] = &[const { String::new() }; 3];
const ARRAY_CONST_BLOCK_STR_SLICE: &[&'static str; 3] = &["hello", "world", "!!!"];

static STATIC_INT: i32 = 10;
static mut STATIC_MUT_INT: u32 = 20;
static mut STATIC_MUT_ARRAY: &'static [String; 3] = &[const { String::new() }; 3];

// Evil because this will evaluate to a value that might need
// to be weirdly represented in the decl file, specifically the
// const tag that i aim to add to all `const` vars.
const EVIL_NESTED_ARRAY_CONST: [[[u32; 2]; 3]; 4] = [[[10; 2]; 3]; 4];

fn main() {
    println!("Hello, world!");
}

fn foo(x: i32) -> i32 {
    if x < 0 {
        return 0;
    }
    x + 1
}

fn bar(y: &[String]) {
    // let x = dep::VISIBLE_FROM_DEP; // only this const is visible, other one stays hidden
    println!("bar func")
}
