use tlua_strings::LuaString;

use crate::{
    NumLike,
    Number,
    OpError,
    StringLike,
    Truthy,
};

pub trait OpName {
    const NAME: &'static str;
}

/// Traits for evaluating anything that looks like an f64 or i64
pub trait NumericOpEval {
    fn evaluate<Lhs, Rhs>(lhs: Lhs, rhs: Rhs) -> Result<Number, OpError>
    where
        Lhs: NumLike,
        Rhs: NumLike;
}

/// Traits for evaluating anything truthy
pub trait BooleanOpEval {
    fn evaluate<Res, Lhs: Truthy + Into<Res>, Rhs: Truthy + Into<Res>>(lhs: Lhs, rhs: Rhs) -> Res;
}

pub trait FloatBinop {
    fn apply_ints(lhs: i64, rhs: i64) -> Number;
    fn apply_floats(lhs: f64, rhs: f64) -> Number;
}

pub trait IntBinop {
    fn apply_ints(lhs: i64, rhs: i64) -> Number;
}

pub trait ComparisonOpEval {
    fn apply_numbers(lhs: Number, rhs: Number) -> bool;

    fn apply_strings<Lhs, Rhs>(lhs: &Lhs, rhs: &Rhs) -> bool
    where
        Lhs: StringLike,
        Rhs: StringLike;

    fn apply_bools(lhs: bool, rhs: bool) -> Result<bool, OpError>;

    fn apply_nils() -> Result<bool, OpError>;
}

pub trait ConcatBinop {
    fn evaluate<Res: From<LuaString>, Lhs: StringLike, Rhs: StringLike>(lhs: Lhs, rhs: Rhs) -> Res;
}
