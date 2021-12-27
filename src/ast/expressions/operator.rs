use enum_dispatch::enum_dispatch;

use crate::ast::expressions::Expression;

#[derive(Debug, PartialEq)]
pub(crate) struct Plus<'chunk> {
    pub lhs: &'chunk Expression<'chunk>,
    pub rhs: &'chunk Expression<'chunk>,
}

#[derive(Debug, PartialEq)]
pub(crate) struct Minus<'chunk> {
    pub lhs: &'chunk Expression<'chunk>,
    pub rhs: &'chunk Expression<'chunk>,
}

#[derive(Debug, PartialEq)]
pub(crate) struct Times<'chunk> {
    pub lhs: &'chunk Expression<'chunk>,
    pub rhs: &'chunk Expression<'chunk>,
}

#[derive(Debug, PartialEq)]
pub(crate) struct Divide<'chunk> {
    pub lhs: &'chunk Expression<'chunk>,
    pub rhs: &'chunk Expression<'chunk>,
}

#[derive(Debug, PartialEq)]
pub(crate) struct IDiv<'chunk> {
    pub lhs: &'chunk Expression<'chunk>,
    pub rhs: &'chunk Expression<'chunk>,
}

#[derive(Debug, PartialEq)]
pub(crate) struct Modulo<'chunk> {
    pub lhs: &'chunk Expression<'chunk>,
    pub rhs: &'chunk Expression<'chunk>,
}

#[derive(Debug, PartialEq)]
pub(crate) struct Exponetiation<'chunk> {
    pub lhs: &'chunk Expression<'chunk>,
    pub rhs: &'chunk Expression<'chunk>,
}

#[derive(Debug, PartialEq)]
pub(crate) struct BitAnd<'chunk> {
    pub lhs: &'chunk Expression<'chunk>,
    pub rhs: &'chunk Expression<'chunk>,
}

#[derive(Debug, PartialEq)]
pub(crate) struct BitOr<'chunk> {
    pub lhs: &'chunk Expression<'chunk>,
    pub rhs: &'chunk Expression<'chunk>,
}

#[derive(Debug, PartialEq)]
pub(crate) struct BitXor<'chunk> {
    pub lhs: &'chunk Expression<'chunk>,
    pub rhs: &'chunk Expression<'chunk>,
}

#[derive(Debug, PartialEq)]
pub(crate) struct ShiftLeft<'chunk> {
    pub lhs: &'chunk Expression<'chunk>,
    pub rhs: &'chunk Expression<'chunk>,
}

#[derive(Debug, PartialEq)]
pub(crate) struct ShiftRight<'chunk> {
    pub lhs: &'chunk Expression<'chunk>,
    pub rhs: &'chunk Expression<'chunk>,
}

#[derive(Debug, PartialEq)]
pub(crate) struct Concat<'chunk> {
    pub lhs: &'chunk Expression<'chunk>,
    pub rhs: &'chunk Expression<'chunk>,
}

#[derive(Debug, PartialEq)]
pub(crate) struct LessThan<'chunk> {
    pub lhs: &'chunk Expression<'chunk>,
    pub rhs: &'chunk Expression<'chunk>,
}

#[derive(Debug, PartialEq)]
pub(crate) struct LessEqual<'chunk> {
    pub lhs: &'chunk Expression<'chunk>,
    pub rhs: &'chunk Expression<'chunk>,
}

#[derive(Debug, PartialEq)]
pub(crate) struct GreaterThan<'chunk> {
    pub lhs: &'chunk Expression<'chunk>,
    pub rhs: &'chunk Expression<'chunk>,
}

#[derive(Debug, PartialEq)]
pub(crate) struct GreaterEqual<'chunk> {
    pub lhs: &'chunk Expression<'chunk>,
    pub rhs: &'chunk Expression<'chunk>,
}

#[derive(Debug, PartialEq)]
pub(crate) struct Equals<'chunk> {
    pub lhs: &'chunk Expression<'chunk>,
    pub rhs: &'chunk Expression<'chunk>,
}

#[derive(Debug, PartialEq)]
pub(crate) struct NotEqual<'chunk> {
    pub lhs: &'chunk Expression<'chunk>,
    pub rhs: &'chunk Expression<'chunk>,
}

#[derive(Debug, PartialEq)]
pub(crate) struct And<'chunk> {
    pub lhs: &'chunk Expression<'chunk>,
    pub rhs: &'chunk Expression<'chunk>,
}

#[derive(Debug, PartialEq)]
pub(crate) struct Or<'chunk> {
    pub lhs: &'chunk Expression<'chunk>,
    pub rhs: &'chunk Expression<'chunk>,
}

#[enum_dispatch]
#[derive(Debug, PartialEq)]
pub(crate) enum BinaryOperator<'chunk> {
    Plus(Plus<'chunk>),
    Minus(Minus<'chunk>),
    Times(Times<'chunk>),
    Divide(Divide<'chunk>),
    IDiv(IDiv<'chunk>),
    Modulo(Modulo<'chunk>),
    Exponetiation(Exponetiation<'chunk>),
    BitAnd(BitAnd<'chunk>),
    BitOr(BitOr<'chunk>),
    BitXor(BitXor<'chunk>),
    ShiftLeft(ShiftLeft<'chunk>),
    ShiftRight(ShiftRight<'chunk>),
    Concat(Concat<'chunk>),
    LessThan(LessThan<'chunk>),
    LessEqual(LessEqual<'chunk>),
    GreaterThan(GreaterThan<'chunk>),
    GreaterEqual(GreaterEqual<'chunk>),
    Equals(Equals<'chunk>),
    NotEqual(NotEqual<'chunk>),
    And(And<'chunk>),
    Or(Or<'chunk>),
}

#[derive(Debug, PartialEq)]
pub(crate) struct Negation<'chunk>(pub &'chunk Expression<'chunk>);

#[derive(Debug, PartialEq)]
pub(crate) struct Not<'chunk>(pub &'chunk Expression<'chunk>);

#[derive(Debug, PartialEq)]
pub(crate) struct Length<'chunk>(pub &'chunk Expression<'chunk>);

#[derive(Debug, PartialEq)]
pub(crate) struct BitNot<'chunk>(pub &'chunk Expression<'chunk>);

#[enum_dispatch]
#[derive(Debug, PartialEq)]
pub(crate) enum UnaryOperator<'chunk> {
    Minus(Negation<'chunk>),
    Not(Not<'chunk>),
    Length(Length<'chunk>),
    BitNot(BitNot<'chunk>),
}
