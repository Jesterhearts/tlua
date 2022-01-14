use crate::{
    binop::{
        traits::BooleanOpEval,
        OpName,
    },
    opcodes::{
        AnyReg,
        Operand,
    },
    Truthy,
};

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct BoolOp<OpTy: BooleanOpEval, RegisterTy> {
    pub lhs: AnyReg<RegisterTy>,
    pub rhs: Operand<RegisterTy>,
    op: OpTy,
}

impl<OpTy, RegisterTy> BooleanOpEval for BoolOp<OpTy, RegisterTy>
where
    OpTy: BooleanOpEval,
{
    fn evaluate<RES, LHS: Truthy + Into<RES>, RHS: Truthy + Into<RES>>(lhs: LHS, rhs: RHS) -> RES {
        OpTy::evaluate(lhs, rhs)
    }
}

impl<OpTy, RegisterTy> From<BoolOp<OpTy, RegisterTy>> for (AnyReg<RegisterTy>, Operand<RegisterTy>)
where
    OpTy: BooleanOpEval,
{
    fn from(val: BoolOp<OpTy, RegisterTy>) -> Self {
        (val.lhs, val.rhs)
    }
}

impl<OpTy, RegisterTy> From<(AnyReg<RegisterTy>, Operand<RegisterTy>)> for BoolOp<OpTy, RegisterTy>
where
    OpTy: BooleanOpEval + Default,
{
    fn from((lhs, rhs): (AnyReg<RegisterTy>, Operand<RegisterTy>)) -> Self {
        Self {
            lhs,
            rhs,
            op: Default::default(),
        }
    }
}

fn evaluate_and<RES, LHS, RHS>(lhs: LHS, rhs: RHS) -> RES
where
    LHS: Truthy + Into<RES>,
    RHS: Truthy + Into<RES>,
{
    if !lhs.as_bool() {
        lhs.into()
    } else {
        rhs.into()
    }
}

fn evaluate_or<RES, LHS, RHS>(lhs: LHS, rhs: RHS) -> RES
where
    LHS: Truthy + Into<RES>,
    RHS: Truthy + Into<RES>,
{
    if lhs.as_bool() {
        lhs.into()
    } else {
        rhs.into()
    }
}

macro_rules! bool_binop_impl {
    ($name:ident => ($lhs:ident : bool, $rhs:ident : bool) => $op:expr $(,)?) => {
        #[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
        pub struct $name;

        impl OpName for $name {
            const NAME: &'static str = stringify!($name);
        }

        impl BooleanOpEval for $name {
            fn evaluate<RES, LHS, RHS>(lhs: LHS, rhs: RHS) -> RES
            where
                LHS: Truthy + Into<RES>,
                RHS: Truthy + Into<RES>,
            {
                let $lhs = lhs;
                let $rhs = rhs;
                $op
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

bool_binop!(And => (lhs: bool, rhs: bool) => evaluate_and(lhs, rhs));
bool_binop!(Or => (lhs: bool, rhs: bool) => evaluate_or(lhs, rhs));
