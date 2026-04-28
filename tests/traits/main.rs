use std::ops::Add;

struct MyStruct<T>(u32, T);

impl<T> Add for MyStruct<T>
where
    T: std::ops::Add<Output=T>,
{
    type Output = MyStruct<T>;

    fn add(self, rhs: Self) -> Self::Output {
        MyStruct(self.0 + rhs.0, self.1 + rhs.1)
    }
}

fn main() { 
    foo(MyStruct(10, 20), MyStruct(30, 40));
}

fn foo(a: MyStruct<u32>, b: MyStruct<u32>) -> MyStruct<u32> {
    a + b
}
