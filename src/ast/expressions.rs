use enum_dispatch::enum_dispatch;

use crate::{
    ast::{
        constant_string::ConstantString,
        prefix_expression::{
            FnCallPrefixExpression,
            VarPrefixExpression,
        },
    },
    values::Nil,
    vm::Number,
};

pub mod function_defs;
pub mod operator;
pub mod tables;

use self::{
    function_defs::FnBody,
    operator::{
        BinaryOperator,
        UnaryOperator,
    },
    tables::TableConstructor,
};

#[derive(Debug, PartialEq)]
pub(crate) struct VarArgs;

#[enum_dispatch]
#[derive(Debug, PartialEq)]
pub(crate) enum Expression<'chunk> {
    Parenthesized(&'chunk Expression<'chunk>),
    Variable(&'chunk VarPrefixExpression<'chunk>),
    FunctionCall(&'chunk FnCallPrefixExpression<'chunk>),
    Nil(Nil),
    Bool(bool),
    Number(Number),
    String(ConstantString),
    FnDef(&'chunk FnBody<'chunk>),
    TableConstructor(TableConstructor<'chunk>),
    VarArgs,
    BinaryOp(BinaryOperator<'chunk>),
    UnaryOp(UnaryOperator<'chunk>),
}

#[cfg(test)]
mod tests {
    use pretty_assertions::Comparison;

    use super::Expression;

    #[test]
    fn sizeof_expr() {
        let left = std::mem::size_of::<Expression>();
        let right = std::mem::size_of::<usize>() * 4;
        if left > right {
            panic!(
                "assertion failed: `(left <= right)`\
                        \n\
                        \n{}:\
                        \n\
                        \n",
                Comparison::new(&left, &right)
            );
        }
    }
}
