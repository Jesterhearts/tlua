use tlua_bytecode::{
    binop::{
        traits::{
            BooleanOpEval,
            OpName,
        },
        BinOpData,
        BoolOpTag,
    },
    Constant,
    OpError,
    Register,
};

use crate::vm::{
    binop::traits::ApplyBinop,
    runtime::value::function::ScopeSet,
};

impl<OpTy> ApplyBinop for BinOpData<BoolOpTag<OpTy>, Register, Constant>
where
    OpTy: OpName + BooleanOpEval,
{
    fn apply(&self, scopes: &mut ScopeSet) -> Result<(), OpError> {
        let lhs = scopes.load(self.lhs);
        let rhs = self.rhs;
        scopes.store(self.lhs, OpTy::evaluate(lhs, rhs));

        Ok(())
    }
}

impl<OpTy> ApplyBinop for BinOpData<BoolOpTag<OpTy>, Register, Register>
where
    OpTy: OpName + BooleanOpEval,
{
    fn apply(&self, scopes: &mut ScopeSet) -> Result<(), OpError> {
        let lhs = scopes.load(self.lhs);
        let rhs = scopes.load(self.rhs);

        scopes.store(self.lhs, OpTy::evaluate(lhs, rhs));

        Ok(())
    }
}
