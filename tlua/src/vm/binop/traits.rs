use tlua_parser::ast::constant_string::ConstantString;

use crate::{
    values::MetaMethod,
    vm::{
        runtime::value::{
            function::ScopeSet,
            Function,
            NumLike,
            Table,
        },
        Number,
        OpError,
    },
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
    fn evaluate(lhs: bool, rhs: bool) -> bool;
}

/// Runtime dispatch trait for binary operations.
pub trait ApplyBinop {
    fn apply(&self, scopes: &mut ScopeSet) -> Result<(), OpError>;
}

pub trait FloatBinop {
    fn apply_ints(lhs: i64, rhs: i64) -> Number;
    fn apply_floats(lhs: f64, rhs: f64) -> Number;

    fn metamethod() -> MetaMethod;
}

pub trait IntBinop {
    fn apply_ints(lhs: i64, rhs: i64) -> Number;

    fn metamethod() -> MetaMethod;
}

pub trait StringLike {
    fn as_bytes(&self) -> &[u8];
}

impl StringLike for ConstantString {
    fn as_bytes(&self) -> &[u8] {
        self.data().as_slice()
    }
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

pub trait CompareBinop: ComparisonOpEval {
    fn apply_tables(lhs: &Table, rhs: &Table) -> Result<bool, OpError>;
    fn apply_functions(lhs: &Function, rhs: &Function) -> Result<bool, OpError>;

    fn metamethod() -> MetaMethod;
}
