use tlua_bytecode::OpError;

use crate::vm::runtime::value::function::ScopeSet;

/// Runtime dispatch trait for binary operations.
pub trait ApplyBinop {
    fn apply(&self, scopes: &mut ScopeSet) -> Result<(), OpError>;
}
