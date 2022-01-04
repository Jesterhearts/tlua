use crate::{
    binop::{
        traits::ComparisonOpEval,
        OpName,
    },
    Number,
    OpError,
    StringLike,
};

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct CompareOp<OpTy, LhsTy, RhsTy> {
    pub lhs: LhsTy,
    pub rhs: RhsTy,
    op: OpTy,
}

impl<OpTy, LhsTy, RhsTy> From<CompareOp<OpTy, LhsTy, RhsTy>> for (LhsTy, RhsTy) {
    fn from(val: CompareOp<OpTy, LhsTy, RhsTy>) -> Self {
        (val.lhs, val.rhs)
    }
}

impl<OpTy, LhsTy, RhsTy> From<(LhsTy, RhsTy)> for CompareOp<OpTy, LhsTy, RhsTy>
where
    OpTy: Default,
{
    fn from((lhs, rhs): (LhsTy, RhsTy)) -> Self {
        Self {
            lhs,
            rhs,
            op: Default::default(),
        }
    }
}

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
        #[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
        pub struct $name;

        impl OpName for $name {
            const NAME: &'static str = stringify!($name);
        }

        impl<RhsTy, LhsTy> ComparisonOpEval for CompareOp<$name, RhsTy, LhsTy> {
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
