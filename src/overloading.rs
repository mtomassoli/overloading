pub trait Trait1 {
    fn method1(&self) -> u32;
}

pub trait Trait2 {
    fn method2(&self) -> String;
}

struct NoImplTrait1 {}
struct NoImplTrait2 {}

impl Trait1 for NoImplTrait1 {
    fn method1(&self) -> u32 {
        unimplemented!()
    }
}

impl Trait2 for NoImplTrait2 {
    fn method2(&self) -> String {
        unimplemented!()
    }
}

static NO_IMPL_TRAIT1: NoImplTrait1 = NoImplTrait1 {};
static NO_IMPL_TRAIT2: NoImplTrait2 = NoImplTrait2 {};

#[derive(Clone, Copy)]
pub struct AsTrait1<'a, T: Trait1>(pub &'a T);

#[derive(Clone, Copy)]
pub struct AsTrait2<'a, T: Trait2>(pub &'a T);

enum AllTypes {
    Type1,
    Type2,
}

trait WithTrait1Or2 {
    const TYPE: AllTypes;
    fn t1(&self) -> &impl Trait1;
    fn t2(&self) -> &impl Trait2;
}

impl<T: Trait1> WithTrait1Or2 for AsTrait1<'_, T> {
    const TYPE: AllTypes = AllTypes::Type1;

    fn t1(&self) -> &impl Trait1 {
        self.0
    }

    fn t2(&self) -> &impl Trait2 {
        &NO_IMPL_TRAIT2
    }
}

impl<T: Trait2> WithTrait1Or2 for AsTrait2<'_, T> {
    const TYPE: AllTypes = AllTypes::Type2;

    fn t1(&self) -> &impl Trait1 {
        &NO_IMPL_TRAIT1
    }

    fn t2(&self) -> &impl Trait2 {
        self.0
    }
}

#[derive(PartialEq)]
pub enum FResult {
    Str(String),
    IntInt(u32, u32),
}

#[allow(private_bounds)]
pub fn f<T1: WithTrait1Or2, T2: WithTrait1Or2>(x: T1, y: T2) -> FResult {
    // NOTE: It's important to ALWAYS check for exhaustiveness.
    match (T1::TYPE, T2::TYPE) {
        (AllTypes::Type2, _) => FResult::Str(x.t2().method2()),
        (_, AllTypes::Type2) => FResult::Str(y.t2().method2()),
        (AllTypes::Type1, AllTypes::Type1) => FResult::IntInt(
            x.t1().method1(), y.t1().method1()),
    }
}

enum XorTypes {
    Types1And2,
    Types2And1,
}

trait PairWithTraits1Xor2 {
    const TYPES: XorTypes;

    fn t12(&self) -> (&impl Trait1, &impl Trait2);
    fn t21(&self) -> (&impl Trait2, &impl Trait1);
}

impl<T1, T2> PairWithTraits1Xor2 for (AsTrait1<'_, T1>, AsTrait2<'_, T2>)
where
    T1: Trait1,
    T2: Trait2,
{
    const TYPES: XorTypes = XorTypes::Types1And2;

    fn t12(&self) -> (&impl Trait1, &impl Trait2) {
        (self.0.0, self.1.0)
    }

    fn t21(&self) -> (&impl Trait2, &impl Trait1) {
        (&NO_IMPL_TRAIT2, &NO_IMPL_TRAIT1)
    }
}

impl<T1, T2> PairWithTraits1Xor2 for (AsTrait2<'_, T1>, AsTrait1<'_, T2>)
where
    T1: Trait2,
    T2: Trait1,
{
    const TYPES: XorTypes = XorTypes::Types2And1;

    fn t12(&self) -> (&impl Trait1, &impl Trait2) {
        (&NO_IMPL_TRAIT1, &NO_IMPL_TRAIT2)
    }

    fn t21(&self) -> (&impl Trait2, &impl Trait1) {
        (self.0.0, self.1.0)
    }
}

#[derive(PartialEq)]
pub enum FXorResult {
    IntStr(u32, String),
    StrInt(String, u32)
}

#[allow(private_bounds)]
pub fn f_xor<P: PairWithTraits1Xor2>(x_y: P) -> FXorResult {
    match P::TYPES {
        XorTypes::Types1And2 => {
            let (x, y) = x_y.t12();
            FXorResult::IntStr(x.method1(), y.method2())
        }
        XorTypes::Types2And1 => {
            let (x, y) = x_y.t21();
            FXorResult::StrInt(x.method2(), y.method1())
        }
    }
}
