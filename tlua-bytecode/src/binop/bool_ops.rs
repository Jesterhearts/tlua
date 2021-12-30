use std::marker::PhantomData;

use derive_more::{
    Deref,
    From,
};

use crate::{
    binop::{
        traits::BooleanOpEval,
        BinOpData,
        OpName,
    },
    Truthy,
};

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct BoolOpTag<OpTy: BooleanOpEval>(PhantomData<OpTy>);

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
        #[derive(Debug, Clone, Copy, PartialEq, From, Deref)]
        pub struct $name<LhsTy, RhsTy>(BinOpData<BoolOpTag<Self>, LhsTy, RhsTy>);

        impl<LhsTy, RhsTy> From<(LhsTy, RhsTy)> for $name<LhsTy, RhsTy> {
            fn from((lhs, rhs): (LhsTy, RhsTy)) -> Self {
                Self(BinOpData {
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
