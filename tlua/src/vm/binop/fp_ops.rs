use tlua_bytecode::{
    binop::{
        traits::{
            FloatBinop,
            NumericOpEval,
            OpName,
        },
        FloatOp,
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

impl<OpTy> ApplyBinop for FloatOp<OpTy, Register>
where
    OpTy: OpName + FloatBinop,
{
    fn apply(&self, scopes: &mut ScopeSet) -> Result<(), OpError> {
        scopes.store(
            self.lhs,
            match scopes.load(self.lhs) {
                Value::Number(n) => Value::Number(Self::evaluate(
                    n,
                    Value::try_from(self.rhs).unwrap_or_else(|reg| scopes.load(reg)),
                )?),
                Value::Table(_) => {
                    todo!("metatables are not supported");
                }
                _ => return Err(OpError::InvalidType { op: OpTy::NAME }),
            },
        );

        Ok(())
    }
}
