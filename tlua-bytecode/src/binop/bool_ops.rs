use crate::{
    binop::{
        traits::BooleanOpEval,
        OpName,
    },
    Truthy,
};

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct BoolOp<OpTy: BooleanOpEval, LhsTy, RhsTy> {
    pub lhs: LhsTy,
    pub rhs: RhsTy,
    op: OpTy,
}

impl<OpTy, LhsTy, RhsTy> BooleanOpEval for BoolOp<OpTy, LhsTy, RhsTy>
where
    OpTy: BooleanOpEval,
{
    fn evaluate<RES, LHS: Truthy + Into<RES>, RHS: Truthy + Into<RES>>(lhs: LHS, rhs: RHS) -> RES {
        OpTy::evaluate(lhs, rhs)
    }
}

impl<OpTy, LhsTy, RhsTy> From<BoolOp<OpTy, LhsTy, RhsTy>> for (LhsTy, RhsTy)
where
    OpTy: BooleanOpEval,
{
    fn from(val: BoolOp<OpTy, LhsTy, RhsTy>) -> Self {
        (val.lhs, val.rhs)
    }
}

impl<OpTy, LhsTy, RhsTy> From<(LhsTy, RhsTy)> for BoolOp<OpTy, LhsTy, RhsTy>
where
    OpTy: BooleanOpEval + Default,
{
    fn from((lhs, rhs): (LhsTy, RhsTy)) -> Self {
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
