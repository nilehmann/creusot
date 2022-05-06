use crate as creusot_contracts;
use crate::logic::*;
use creusot_contracts_proc::*;

#[creusot::builtins = "map.Map.map"]
pub struct Mapping<A, B>(std::marker::PhantomData<(A, B)>);

impl<A, B> Mapping<A, B> {
    #[trusted]
    #[logic]
    #[creusot::builtins = "map.Map.get"]
    pub fn get(self, _: A) -> B {
        absurd
    }

    #[trusted]
    #[logic]
    #[creusot::builtins = "map.Map.set"]
    pub fn set(self, _: A, _: B) -> Self {
        absurd
    }

    #[trusted]
    #[logic]
    #[creusot::builtins = "map.Const.const"]
    pub fn cst(_: B) -> Self {
        absurd
    }
}

impl<A, B> EqLogic for Mapping<A, B> {
    #[trusted]
    #[logic]
    #[creusot::builtins = "map.MapExt.(==)"]
    fn log_eq(self, _: Self) -> bool {
        absurd
    }

    #[trusted]
    #[predicate]
    fn log_ne(self, _: Self) -> bool {
        absurd
    }

    #[trusted]
    #[logic]
    fn eq_ne(_: Self, _: Self) {
        absurd
    }

    // lemmas below are marked trusted, until Creusot provides a way
    // to prove them

    #[trusted]
    #[logic]
    fn refl(_: Self) {}

    #[trusted]
    #[logic]
    fn symmetry(_: Self, _: Self) {}

    #[trusted]
    #[logic]
    fn transitivity(_: Self, _: Self, _: Self) {}
}