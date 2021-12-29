use tlua_bytecode::{
    binop::traits::ComparisonOpEval,
    OpError,
};

use crate::{
    values::MetaMethod,
    vm::runtime::value::{
        function::ScopeSet,
        Function,
        Table,
    },
};

/// Runtime dispatch trait for binary operations.
pub trait ApplyBinop {
    fn apply(&self, scopes: &mut ScopeSet) -> Result<(), OpError>;
}

pub trait CompareBinop: ComparisonOpEval {
    fn apply_tables(lhs: &Table, rhs: &Table) -> Result<bool, OpError>;
    fn apply_functions(lhs: &Function, rhs: &Function) -> Result<bool, OpError>;

    fn metamethod() -> MetaMethod;
}
