mod overloading;

use overloading::{f, f_xor, AsTrait1, AsTrait2, FResult, FXorResult, Trait1, Trait2};

struct MyType1;
struct MyType2;
struct MyType3;

impl Trait1 for MyType1 {
    fn method1(&self) -> u32 {
        7
    }
}

impl Trait2 for MyType2 {
    fn method2(&self) -> String {
        "asd".into()
    }
}

impl Trait1 for MyType3 {
    fn method1(&self) -> u32 {
        3
    }
}

fn main() {
    let t1 = MyType1;
    let t2 = MyType2;
    let t3 = MyType3;

    assert!(f(AsTrait1(&t1), AsTrait1(&t3)) == FResult::IntInt(7, 3));
    assert!(f(AsTrait1(&t1), AsTrait1(&t1)) == FResult::IntInt(7, 7));
    assert!(f(AsTrait1(&t1), AsTrait2(&t2)) == FResult::Str("asd".into()));
    assert!(f(AsTrait2(&t2), AsTrait1(&t1)) == FResult::Str("asd".into()));

    // f_xor((AsTrait1(&t1), AsTrait1(&t3)));  // trait bound not satisfied
    assert!(f_xor((AsTrait1(&t1), AsTrait2(&t2))) == FXorResult::IntStr(7, "asd".into()));
    assert!(f_xor((AsTrait2(&t2), AsTrait1(&t3))) == FXorResult::StrInt("asd".into(), 3));
    // f_xor((AsTrait2(&t2), AsTrait2(&t2)));  // trait bound not satisfied

    println!("All OK!");
}
