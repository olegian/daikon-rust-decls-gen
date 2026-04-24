const INT_CONST: u32 = 10 + 10;
const ARRAY_CONST: [isize; 3] = [100; 3];

const fn square(x: u32) -> u32 {
    x * x
}
const FN_INITED_CONST: [u32; 3] = [square(1), square(2), square(3)];
const CONST_BLOCK: [String; 3] = [const { String::new() }; 3];

static STATIC_INT: i32 = 10;
static mut STATIC_MUT_INT: u32 = 20;
static mut STATIC_MUT_ARRAY: [String; 3] = [const { String::new() }; 3];

// Evil because this will evaluate to a value that might need
// to be specially represented in the decl file, specifically the 
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
