use crate::{
    ast::{
        expressions::Expression,
        identifiers::Ident,
    },
    list::List,
};

pub mod function_calls;

use self::function_calls::FnArgs;

#[derive(Debug, PartialEq)]
pub enum HeadAtom<'chunk> {
    Name(Ident),
    Parenthesized(&'chunk Expression<'chunk>),
}

#[derive(Debug, PartialEq)]
pub enum VarAtom<'chunk> {
    Name(Ident),
    IndexOp(Expression<'chunk>),
}

#[derive(Debug, PartialEq)]
pub enum FunctionAtom<'chunk> {
    Call(FnArgs<'chunk>),
    MethodCall { name: Ident, args: FnArgs<'chunk> },
}

#[derive(Debug, PartialEq)]
pub enum PrefixAtom<'chunk> {
    Var(VarAtom<'chunk>),
    Function(FunctionAtom<'chunk>),
}

#[derive(Debug, PartialEq)]
pub enum VarPrefixExpression<'chunk> {
    Name(Ident),
    TableAccess {
        head: HeadAtom<'chunk>,
        middle: List<'chunk, PrefixAtom<'chunk>>,
        last: &'chunk VarAtom<'chunk>,
    },
}

#[derive(Debug, PartialEq)]
pub enum FnCallPrefixExpression<'chunk> {
    Call {
        head: HeadAtom<'chunk>,
        args: FunctionAtom<'chunk>,
    },
    CallPath {
        head: HeadAtom<'chunk>,
        middle: List<'chunk, PrefixAtom<'chunk>>,
        last: FunctionAtom<'chunk>,
    },
}

#[derive(Debug, PartialEq)]
pub enum PrefixExpression<'chunk> {
    Variable(VarPrefixExpression<'chunk>),
    FnCall(FnCallPrefixExpression<'chunk>),
    Parenthesized(&'chunk Expression<'chunk>),
}
