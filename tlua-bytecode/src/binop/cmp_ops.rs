use std::marker::PhantomData;

use derive_more::{
    Deref,
    From,
};

use crate::{
    binop::{
        traits::ComparisonOpEval,
        BinOpData,
        OpName,
    },
    Number,
    OpError,
    StringLike,
};

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct CompareOpTag<OpTy: ComparisonOpEval>(PhantomData<OpTy>);

macro_rules! comparison_binop_impl {
    (
        $name:ident =>
        {
            ($lhs_num:ident : num, $rhs_num:ident : num) => $when_num:expr,
            ($lhs_string:ident : string, $rhs_string:ident : string) => $when_string:expr,
            ($lhs_bool:ident : bool, $rhs_bool:ident : bool) => $when_bool:expr,
            (nil,nil) => $when_nil:expr,
            ($lhs_table:ident : table, $rhs_table:ident : table) => $when_table:expr,
            ($lhs_func:ident : func, $rhs_func:ident : func) => $when_func:expr $(,)?
        }
    ) => {
        #[derive(Debug, Clone, Copy, PartialEq, From, Deref)]
        pub struct $name<LhsTy, RhsTy>(BinOpData<CompareOpTag<Self>, LhsTy, RhsTy>);

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

        impl<LhsTy, RhsTy> ComparisonOpEval for $name<LhsTy, RhsTy> {
            fn apply_numbers(lhs: Number, rhs: Number) -> bool {
                let $lhs_num = lhs;
                let $rhs_num = rhs;

                $when_num
            }

            fn apply_strings<LHS, RHS>(lhs: &LHS, rhs: &RHS) -> bool
            where
                LHS: StringLike,
                RHS: StringLike,
            {
                let $lhs_string = lhs;
                let $rhs_string = rhs;

                $when_string
            }

            fn apply_bools(lhs: bool, rhs: bool) -> Result<bool, OpError> {
                let $lhs_bool = lhs;
                let $rhs_bool = rhs;

                $when_bool
            }

            fn apply_nils() -> Result<bool, OpError> {
                $when_nil
            }
        }
    };
}

macro_rules! comparison_binop {
    (
        $name:ident =>
        {
            ($lhs_num:ident : num, $rhs_num:ident : num) => $when_num:expr,
            ($lhs_string:ident : string, $rhs_string:ident : string) => $when_string:expr,
            ($lhs_bool:ident : bool, $rhs_bool:ident : bool) => $when_bool:expr,
            (nil,nil) => $when_nil:expr,
            ($lhs_table:ident : table, $rhs_table:ident : table) => $when_table:expr,
            ($lhs_func:ident : func, $rhs_func:ident : func) => $when_func:expr $(,)?
        }
    ) => {
        comparison_binop_impl! { $name => {
            ($lhs_num : num, $rhs_num : num) => $when_num,
            ($lhs_string : string, $rhs_string : string) => $when_string,
            ($lhs_bool : bool, $rhs_bool : bool) => $when_bool,
            (nil, nil) => $when_nil,
            ($lhs_table : table, $rhs_table : table) => $when_table,
            ($lhs_func : func, $rhs_func : func) => $when_func
        } }

        paste::paste! {comparison_binop_impl! { [< $name Indirect>] => {
            ($lhs_num : num, $rhs_num : num) => $when_num,
            ($lhs_string : string, $rhs_string : string) => $when_string,
            ($lhs_bool : bool, $rhs_bool : bool) => $when_bool,
            (nil, nil) => $when_nil,
            ($lhs_table : table, $rhs_table : table) => $when_table,
            ($lhs_func : func, $rhs_func : func) => $when_func
        }}}
    };
}

// TODO: metatables
comparison_binop!(LessThan => {
    (lhs: num, rhs: num) => lhs < rhs,
    (lhs: string, rhs: string) => lhs.as_bytes() < rhs.as_bytes(),
    (_lhs: bool, _rhs: bool) => Err(OpError::DuoCmpErr{type_name: "bool"}),
    (nil, nil) => Err(OpError::DuoCmpErr{type_name: "nil"}),
    (lhs: table, rhs: table) => Err(OpError::DuoCmpErr{type_name: "table"}),
    (lhs: func, rhs: func) => Err(OpError::DuoCmpErr{type_name: "func"}),
});

comparison_binop!(LessEqual => {
    (lhs: num, rhs: num) => lhs <= rhs,
    (lhs: string, rhs: string) => lhs.as_bytes() <= rhs.as_bytes(),
    (_lhs: bool, _rhs: bool) => Err(OpError::DuoCmpErr{type_name: "bool"}),
    (nil, nil) => Err(OpError::DuoCmpErr{type_name: "nil"}),
    (lhs: table, rhs: table) => Err(OpError::DuoCmpErr{type_name: "table"}),
    (lhs: func, rhs: func) => Err(OpError::DuoCmpErr{type_name: "func"}),
});

comparison_binop!(GreaterThan => {
    (lhs: num, rhs: num) => lhs > rhs,
    (lhs: string, rhs: string) => lhs.as_bytes() > rhs.as_bytes(),
    (_lhs: bool, _rhs: bool) => Err(OpError::DuoCmpErr{type_name: "bool"}),
    (nil, nil) => Err(OpError::DuoCmpErr{type_name: "nil"}),
    (lhs: table, rhs: table) => Err(OpError::DuoCmpErr{type_name: "table"}),
    (lhs: func, rhs: func) => Err(OpError::DuoCmpErr{type_name: "func"}),
});

comparison_binop!(GreaterEqual => {
    (lhs: num, rhs: num) => lhs >= rhs,
    (lhs: string, rhs: string) => lhs.as_bytes() >= rhs.as_bytes(),
    (_lhs: bool, _rhs: bool) => Err(OpError::DuoCmpErr{type_name: "bool"}),
    (nil, nil) => Err(OpError::DuoCmpErr{type_name: "nil"}),
    (lhs: table, rhs: table) => Err(OpError::DuoCmpErr{type_name: "table"}),
    (lhs: func, rhs: func) => Err(OpError::DuoCmpErr{type_name: "func"}),
});

comparison_binop!(Equals => {
    (lhs: num, rhs: num) => lhs == rhs,
    (lhs: string, rhs: string) => lhs.as_bytes() == rhs.as_bytes(),
    (lhs: bool, rhs: bool) => Ok(lhs == rhs),
    (nil, nil) => Ok(true),
    (lhs: table, rhs: table) => Ok(lhs == rhs),
    (lhs: func, rhs: func) => Ok(lhs == rhs)
});

comparison_binop!(NotEqual => {
    (lhs: num, rhs: num) => lhs != rhs,
    (lhs: string, rhs: string) => lhs.as_bytes() != rhs.as_bytes(),
    (lhs: bool, rhs: bool) => Ok(lhs != rhs),
    (nil, nil) => Ok(true),
    (lhs: table, rhs: table) => Ok(lhs != rhs),
    (lhs: func, rhs: func) => Ok(lhs != rhs)
});
