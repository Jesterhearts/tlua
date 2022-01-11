use tlua_bytecode::{
    binop::{
        traits::{
            BooleanOpEval,
            OpName,
        },
        BoolOp,
    },
    OpError,
    Register,
};

use crate::vm::{
    binop::traits::ApplyBinop,
    runtime::{
        value::function::ScopeSet,
        Value,
    },
};

impl<OpTy> ApplyBinop for BoolOp<OpTy, Register>
where
    OpTy: OpName + BooleanOpEval,
{
    fn apply(&self, scopes: &mut ScopeSet) -> Result<(), OpError> {
        let lhs = scopes.load(self.lhs);
        let rhs = Value::try_from(self.rhs).unwrap_or_else(|reg| scopes.load(reg));

        scopes.store(self.lhs, OpTy::evaluate(lhs, rhs));

        Ok(())
    }
}
