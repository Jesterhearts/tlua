use crate::ast::expressions::Expression;

#[derive(Debug, PartialEq)]
pub struct Plus<'chunk> {
    pub lhs: &'chunk Expression<'chunk>,
    pub rhs: &'chunk Expression<'chunk>,
}

#[derive(Debug, PartialEq)]
pub struct Minus<'chunk> {
    pub lhs: &'chunk Expression<'chunk>,
    pub rhs: &'chunk Expression<'chunk>,
}

#[derive(Debug, PartialEq)]
pub struct Times<'chunk> {
    pub lhs: &'chunk Expression<'chunk>,
    pub rhs: &'chunk Expression<'chunk>,
}

#[derive(Debug, PartialEq)]
pub struct Divide<'chunk> {
    pub lhs: &'chunk Expression<'chunk>,
    pub rhs: &'chunk Expression<'chunk>,
}

#[derive(Debug, PartialEq)]
pub struct IDiv<'chunk> {
    pub lhs: &'chunk Expression<'chunk>,
    pub rhs: &'chunk Expression<'chunk>,
}

#[derive(Debug, PartialEq)]
pub struct Modulo<'chunk> {
    pub lhs: &'chunk Expression<'chunk>,
    pub rhs: &'chunk Expression<'chunk>,
}

#[derive(Debug, PartialEq)]
pub struct Exponetiation<'chunk> {
    pub lhs: &'chunk Expression<'chunk>,
    pub rhs: &'chunk Expression<'chunk>,
}

#[derive(Debug, PartialEq)]
pub struct BitAnd<'chunk> {
    pub lhs: &'chunk Expression<'chunk>,
    pub rhs: &'chunk Expression<'chunk>,
}

#[derive(Debug, PartialEq)]
pub struct BitOr<'chunk> {
    pub lhs: &'chunk Expression<'chunk>,
    pub rhs: &'chunk Expression<'chunk>,
}

#[derive(Debug, PartialEq)]
pub struct BitXor<'chunk> {
    pub lhs: &'chunk Expression<'chunk>,
    pub rhs: &'chunk Expression<'chunk>,
}

#[derive(Debug, PartialEq)]
pub struct ShiftLeft<'chunk> {
    pub lhs: &'chunk Expression<'chunk>,
    pub rhs: &'chunk Expression<'chunk>,
}

#[derive(Debug, PartialEq)]
pub struct ShiftRight<'chunk> {
    pub lhs: &'chunk Expression<'chunk>,
    pub rhs: &'chunk Expression<'chunk>,
}

#[derive(Debug, PartialEq)]
pub struct Concat<'chunk> {
    pub lhs: &'chunk Expression<'chunk>,
    pub rhs: &'chunk Expression<'chunk>,
}

#[derive(Debug, PartialEq)]
pub struct LessThan<'chunk> {
    pub lhs: &'chunk Expression<'chunk>,
    pub rhs: &'chunk Expression<'chunk>,
}

#[derive(Debug, PartialEq)]
pub struct LessEqual<'chunk> {
    pub lhs: &'chunk Expression<'chunk>,
    pub rhs: &'chunk Expression<'chunk>,
}

#[derive(Debug, PartialEq)]
pub struct GreaterThan<'chunk> {
    pub lhs: &'chunk Expression<'chunk>,
    pub rhs: &'chunk Expression<'chunk>,
}

#[derive(Debug, PartialEq)]
pub struct GreaterEqual<'chunk> {
    pub lhs: &'chunk Expression<'chunk>,
    pub rhs: &'chunk Expression<'chunk>,
}

#[derive(Debug, PartialEq)]
pub struct Equals<'chunk> {
    pub lhs: &'chunk Expression<'chunk>,
    pub rhs: &'chunk Expression<'chunk>,
}

#[derive(Debug, PartialEq)]
pub struct NotEqual<'chunk> {
    pub lhs: &'chunk Expression<'chunk>,
    pub rhs: &'chunk Expression<'chunk>,
}

#[derive(Debug, PartialEq)]
pub struct And<'chunk> {
    pub lhs: &'chunk Expression<'chunk>,
    pub rhs: &'chunk Expression<'chunk>,
}

#[derive(Debug, PartialEq)]
pub struct Or<'chunk> {
    pub lhs: &'chunk Expression<'chunk>,
    pub rhs: &'chunk Expression<'chunk>,
}

#[derive(Debug, PartialEq)]
pub enum BinaryOperator<'chunk> {
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
pub struct Negation<'chunk>(pub &'chunk Expression<'chunk>);

#[derive(Debug, PartialEq)]
pub struct Not<'chunk>(pub &'chunk Expression<'chunk>);

#[derive(Debug, PartialEq)]
pub struct Length<'chunk>(pub &'chunk Expression<'chunk>);

#[derive(Debug, PartialEq)]
pub struct BitNot<'chunk>(pub &'chunk Expression<'chunk>);

#[derive(Debug, PartialEq)]
pub enum UnaryOperator<'chunk> {
    Minus(Negation<'chunk>),
    Not(Not<'chunk>),
    Length(Length<'chunk>),
    BitNot(BitNot<'chunk>),
}
