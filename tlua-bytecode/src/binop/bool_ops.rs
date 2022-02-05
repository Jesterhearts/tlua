use derive_more::From;

use crate::{
    binop::{
        traits::BooleanOpEval,
        OpName,
    },
    AnonymousRegister,
    Truthy,
};

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
        #[derive(Clone, Copy, PartialEq, Eq, From)]
        pub struct $name {
            pub dst: AnonymousRegister,
            pub lhs: AnonymousRegister,
            pub rhs: AnonymousRegister,
        }

        impl ::std::fmt::Debug for $name {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                write!(
                    f,
                    "{} {:?} {:?} {:?}",
                    Self::NAME,
                    self.dst,
                    self.lhs,
                    self.rhs
                )
            }
        }

        impl OpName for $name {
            const NAME: &'static str = paste::paste! { stringify!([< $name:snake >])};
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
    };
}

bool_binop!(And => (lhs: bool, rhs: bool) => evaluate_and(lhs, rhs));
bool_binop!(Or => (lhs: bool, rhs: bool) => evaluate_or(lhs, rhs));
