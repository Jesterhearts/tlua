use derive_more::{
    Deref,
    From,
};

use crate::vm::{
    binop::{
        traits::{
            ApplyBinop,
            BooleanOpEval,
        },
        BinOp,
        OpName,
    },
    runtime::value::function::ScopeSet,
    Constant,
    OpError,
    Register,
};

macro_rules! bool_binop_impl {
    ($name:ident => ($lhs:ident : bool, $rhs:ident : bool) => $op:expr $(,)?) => {
        #[derive(Debug, Clone, Copy, PartialEq, From, Deref)]
        pub struct $name<LhsTy, RhsTy>(BinOp<Self, LhsTy, RhsTy>);

        impl<LhsTy, RhsTy> From<(LhsTy, RhsTy)> for $name<LhsTy, RhsTy> {
            fn from((lhs, rhs): (LhsTy, RhsTy)) -> Self {
                Self(BinOp {
                    lhs,
                    rhs,
                    _tag: Default::default(),
                })
            }
        }

        impl<LhsTy, RhsTy> OpName for $name<LhsTy, RhsTy> {
            const NAME: &'static str = stringify!($name);
        }

        impl<Lhs, Rhs> BooleanOpEval for $name<Lhs, Rhs> {
            fn evaluate($lhs: bool, $rhs: bool) -> bool {
                $op
            }
        }

        impl ApplyBinop for $name<Register, Constant> {
            fn apply(&self, scopes: &mut ScopeSet) -> Result<(), OpError> {
                let $lhs = scopes.load(self.0.lhs).as_bool();
                let $rhs = self.0.rhs.as_bool();

                scopes.store(self.0.lhs, Self::evaluate($lhs, $rhs).into());

                Ok(())
            }
        }

        impl ApplyBinop for $name<Register, Register> {
            fn apply(&self, scopes: &mut ScopeSet) -> Result<(), OpError> {
                let $lhs = scopes.load(self.0.lhs).as_bool();
                let $rhs = scopes.load(self.0.rhs).as_bool();

                scopes.store(self.0.lhs, Self::evaluate($lhs, $rhs).into());

                Ok(())
            }
        }
    };
}

macro_rules! bool_binop {
    ($name:ident => ($lhs:ident : bool, $rhs:ident : bool) => $op:expr $(,)?) => {
        bool_binop_impl! { $name => ($lhs : bool, $rhs : bool) => $op }

        paste::paste! { bool_binop_impl!{ [< $name Indirect >] => ($lhs :
        bool, $rhs : bool) => $op }}
    };
}

// TODO(lang-5.4): This is wrong, see https://www.lua.org/manual/5.4/manual.html#3.4.5
bool_binop!(And => (lhs: bool, rhs: bool) => lhs && rhs);
bool_binop!(Or => (lhs: bool, rhs: bool) => lhs || rhs);
