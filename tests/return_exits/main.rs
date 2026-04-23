fn main() {
    println!("Hello, world!");
}

fn void_explicit(x: u32) {
    println!("fn: void_explicit");
    return;
}

fn void_implicit(x: u32) {
    println!("fn: void_implicit")
}

fn void_explicit_multiple(x: u32) {
    println!("fn: void_explicit_multiple");
    if x == 0 {
        return;
    }

    if x == 0 {
        return;
    } else {
        return;
    };
}

fn void_implicit_multiple(x: u32) {
    println!("fn: void_implicit_multiple");
    if x == 0 {
        return;
    }

    if x == 0 {
        return;
    }

    // there is also technically a return statement here
}

fn value_explicit(x: u32) -> u32 {
    println!("fn: value_explicit");
    return x;
}

fn value_implicit(x: u32) -> u32 {
    println!("fn: value_implicit");
    x
}

fn value_explicit_multiple(x: u32) -> u32 {
    println!("fn: value_explicit_multiple");
    if x == 0 {
        return x;
    }

    if x == 0 {
        return x;
    } else {
        return x;
    };
}

fn value_implicit_multiple(x: u32) -> u32 {
    println!("fn: value_implicit_multiple");
    if x == 0 {
        return x;
    }

    if x == 0 {
        return x;
    }

    x
}
