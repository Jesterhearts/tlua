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
    fn evaluate<LHS, RHS>(lhs: LHS, rhs: RHS) -> Result<Number, OpError>
    where
        LHS: NumLike,
        RHS: NumLike;
}

/// Traits for evaluating anything truthy
pub trait BooleanOpEval {
    fn evaluate<RES, LHS: Truthy + Into<RES>, RHS: Truthy + Into<RES>>(lhs: LHS, rhs: RHS) -> RES;
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

    fn apply_strings<LHS, RHS>(lhs: &LHS, rhs: &RHS) -> bool
    where
        LHS: StringLike,
        RHS: StringLike;

    fn apply_bools(lhs: bool, rhs: bool) -> Result<bool, OpError>;

    fn apply_nils() -> Result<bool, OpError>;
}
