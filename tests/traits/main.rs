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

enum MyEnum<T> { V1(T), V2(T, T) }
impl std::ops::Add for MyEnum<u32> {
    type Output = MyEnum<u32>;

    fn add(self, rhs: Self) -> Self::Output {
        match (self, rhs) {
            (MyEnum::V1(a), MyEnum::V1(c)) => MyEnum::V1(a + c),
            (MyEnum::V1(a), MyEnum::V2(c, d)) => MyEnum::V2(a + c, d),
            (MyEnum::V2(a, b), MyEnum::V1(c)) => MyEnum::V2(a + c, b),
            (MyEnum::V2(a, b), MyEnum::V2(c, d)) => MyEnum::V2(a + c, b + d),
        }
    }
}


fn main() { 
    foo(MyStruct(10, 20), MyStruct(30, 40));
}

fn foo(a: MyStruct<u32>, b: MyStruct<u32>) -> MyStruct<u32> {
    a + b
}
