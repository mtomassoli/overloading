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

enum AllTraits {
    Trait1,
    Trait2,
}

trait AsTrait1Or2 {
    const TRAIT: AllTraits;
    fn t1(&self) -> &impl Trait1;
    fn t2(&self) -> &impl Trait2;
}

impl<T: Trait1> AsTrait1Or2 for AsTrait1<'_, T> {
    const TRAIT: AllTraits = AllTraits::Trait1;

    fn t1(&self) -> &impl Trait1 {
        self.0
    }

    fn t2(&self) -> &impl Trait2 {
        &NO_IMPL_TRAIT2
    }
}

impl<T: Trait2> AsTrait1Or2 for AsTrait2<'_, T> {
    const TRAIT: AllTraits = AllTraits::Trait2;

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
pub fn f<T1: AsTrait1Or2, T2: AsTrait1Or2>(x: T1, y: T2) -> FResult {
    // NOTE: It's important to ALWAYS check for exhaustiveness.
    match (T1::TRAIT, T2::TRAIT) {
        (AllTraits::Trait2, _) => FResult::Str(x.t2().method2()),
        (_, AllTraits::Trait2) => FResult::Str(y.t2().method2()),
        (AllTraits::Trait1, AllTraits::Trait1) => FResult::IntInt(
            x.t1().method1(), y.t1().method1()),
    }
}

enum XorTraits {
    Traits1And2,
    Traits2And1,
}

trait PairAsTraits1Xor2 {
    const TRAITS: XorTraits;

    fn t12(&self) -> (&impl Trait1, &impl Trait2);
    fn t21(&self) -> (&impl Trait2, &impl Trait1);
}

impl<T1, T2> PairAsTraits1Xor2 for (AsTrait1<'_, T1>, AsTrait2<'_, T2>)
where
    T1: Trait1,
    T2: Trait2,
{
    const TRAITS: XorTraits = XorTraits::Traits1And2;

    fn t12(&self) -> (&impl Trait1, &impl Trait2) {
        (self.0.0, self.1.0)
    }

    fn t21(&self) -> (&impl Trait2, &impl Trait1) {
        (&NO_IMPL_TRAIT2, &NO_IMPL_TRAIT1)
    }
}

impl<T1, T2> PairAsTraits1Xor2 for (AsTrait2<'_, T1>, AsTrait1<'_, T2>)
where
    T1: Trait2,
    T2: Trait1,
{
    const TRAITS: XorTraits = XorTraits::Traits2And1;

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
pub fn f_xor<P: PairAsTraits1Xor2>(x_y: P) -> FXorResult {
    match P::TRAITS {
        XorTraits::Traits1And2 => {
            let (x, y) = x_y.t12();
            FXorResult::IntStr(x.method1(), y.method2())
        }
        XorTraits::Traits2And1 => {
            let (x, y) = x_y.t21();
            FXorResult::StrInt(x.method2(), y.method1())
        }
    }
}
