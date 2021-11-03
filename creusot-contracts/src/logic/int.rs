use super::model::*;
use crate::logic::well_founded::*;
use creusot_contracts_proc::*;
use std::ops::*;

#[rustc_diagnostic_item = "creusot_int"]
pub struct Int;

impl WellFounded for Int {}

impl Int {
    #[trusted]
    #[logic]
    #[creusot::builtins = "int.Power.power"]
    pub fn pow(self, _: Int) -> Int {
        std::process::abort()
    }
}

impl PartialEq for Int {
    #[creusot::no_translate]
    #[rustc_diagnostic_item = "eq_int"]
    fn eq(&self, _: &Int) -> bool {
        panic!()
    }

    #[creusot::no_translate]
    #[rustc_diagnostic_item = "ne_int"]
    fn ne(&self, _: &Int) -> bool {
        panic!()
    }
}

impl PartialOrd for Int {
    #[creusot::no_translate]
    #[rustc_diagnostic_item = "le_int"]
    fn le(&self, _: &Int) -> bool {
        panic!()
    }

    #[creusot::no_translate]
    #[rustc_diagnostic_item = "ge_int"]
    fn ge(&self, _: &Int) -> bool {
        panic!()
    }

    #[creusot::no_translate]
    #[rustc_diagnostic_item = "lt_int"]
    fn lt(&self, _: &Int) -> bool {
        panic!()
    }

    #[creusot::no_translate]
    #[rustc_diagnostic_item = "gt_int"]
    fn gt(&self, _: &Int) -> bool {
        panic!()
    }

    #[creusot::no_translate]
    fn partial_cmp(&self, _: &Self) -> Option<std::cmp::Ordering> {
        panic!()
    }
}

impl From<u32> for Int {
    #[logic]
    #[trusted]
    #[rustc_diagnostic_item = "u32_to_int"]
    #[creusot::builtins = "mach.int.UInt32.to_int"]
    fn from(_: u32) -> Self {
        panic!()
    }
}
impl Model for u32 {
    type ModelTy = Int;
    #[logic]
    #[rustc_diagnostic_item = "u32_model"]
    #[creusot::builtins = "mach.int.UInt32.to_int"]
    fn model(self) -> Self::ModelTy {
        Int::from(self)
    }
}

impl From<i32> for Int {
    #[logic]
    #[trusted]
    #[rustc_diagnostic_item = "i32_to_int"]
    #[creusot::builtins = "mach.int.Int32.to_int"]
    fn from(_: i32) -> Self {
        panic!()
    }
}
impl Model for i32 {
    type ModelTy = Int;
    #[logic]
    #[rustc_diagnostic_item = "i32_model"]
    #[creusot::builtins = "mach.int.Int32.to_int"]
    fn model(self) -> Self::ModelTy {
        Int::from(self)
    }
}

impl From<usize> for Int {
    #[logic]
    #[trusted]
    #[rustc_diagnostic_item = "usize_to_int"]
    #[creusot::builtins = "mach.int.UInt64.to_int"]
    fn from(_: usize) -> Self {
        panic!()
    }
}
impl Model for usize {
    type ModelTy = Int;
    #[logic]
    #[rustc_diagnostic_item = "usize_model"]
    #[creusot::builtins = "mach.int.UInt64.to_int"]
    fn model(self) -> Self::ModelTy {
        Int::from(self)
    }
}

impl From<isize> for Int {
    #[logic]
    #[trusted]
    #[rustc_diagnostic_item = "isize_to_int"]
    fn from(_: isize) -> Self {
        panic!()
    }
}
impl Model for isize {
    type ModelTy = Int;
    #[logic]
    #[rustc_diagnostic_item = "isize_model"]
    #[creusot::builtins = "mach.int.Int64.to_int"]
    fn model(self) -> Self::ModelTy {
        Int::from(self)
    }
}

impl Add<Int> for Int {
    type Output = Int;
    #[creusot::no_translate]
    #[rustc_diagnostic_item = "add_int"]
    fn add(self, _: Int) -> Self {
        panic!()
    }
}

impl Sub<Int> for Int {
    type Output = Int;
    #[creusot::no_translate]
    #[rustc_diagnostic_item = "sub_int"]
    fn sub(self, _: Int) -> Self {
        panic!()
    }
}

impl Mul<Int> for Int {
    type Output = Int;
    #[creusot::no_translate]
    #[rustc_diagnostic_item = "mul_int"]
    fn mul(self, _: Int) -> Self {
        panic!()
    }
}

impl Div<Int> for Int {
    type Output = Int;
    #[creusot::no_translate]
    #[rustc_diagnostic_item = "div_int"]
    fn div(self, _: Int) -> Self {
        panic!()
    }
}

impl Rem<Int> for Int {
    type Output = Int;
    #[creusot::no_translate]
    #[rustc_diagnostic_item = "rem_int"]
    fn rem(self, _: Int) -> Self {
        panic!()
    }
}

impl Neg for Int {
    type Output = Int;
    #[creusot::no_translate]
    #[rustc_diagnostic_item = "neg_int"]
    fn neg(self) -> Self {
        panic!()
    }
}