use nom::{
    branch::alt,
    bytes::complete::tag,
    combinator::{
        map,
        not,
        opt,
    },
    sequence::{
        pair,
        preceded,
        tuple,
    },
};

use crate::{
    expressions::{
        parse_non_op_expr,
        Expression,
    },
    lua_whitespace0,
    lua_whitespace1,
    ASTAllocator,
    ParseResult,
    Span,
};

#[cfg(test)]
mod tests;

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

enum BinOpParseResult<'chunk> {
    NoMatch(Expression<'chunk>),
    Matched(BinaryOperator<'chunk>),
}

/// This is the top of our precedence tree.
/// We first try to match a non-operator expression since we know that would be
/// a leaf. If we attempted to match any expression we would recurse infinitely.
/// Since we reach this node attempting to parse a unary operator, we don't have
/// to worry about handling that case.
/// If we aren't looking at an exponentation expression, we just return whatever
/// leaf we found.
pub fn parse_exp_expr<'src, 'chunk>(
    input: Span<'src>,
    alloc: &'chunk ASTAllocator,
) -> ParseResult<'src, Expression<'chunk>> {
    parse_right_assoc_binop(
        input,
        |input| parse_non_op_expr(input, alloc),
        opt(preceded(pair(tag("^"), lua_whitespace0), |input| {
            parse_non_op_expr(input, alloc)
        })),
        |lhs, rhs| {
            BinaryOperator::Exponetiation(Exponetiation {
                lhs: alloc.alloc(lhs),
                rhs: alloc.alloc(rhs),
            })
        },
    )
}

/// This one is a little weird since the rest of the tree are binary operators.
/// We want to try to match an (optional) UnaryOp followed by an exponentiation,
/// since that will cause the exponentiation operator to be parsed as a full
/// expression first. If we don't find a unary operator, we just return whatever
/// the result of parsing exponentation is.
pub fn parse_unop_expr<'src, 'chunk>(
    input: Span<'src>,
    alloc: &'chunk ASTAllocator,
) -> ParseResult<'src, Expression<'chunk>> {
    alt((
        map(
            preceded(pair(tag("not"), lua_whitespace1), |input| {
                parse_exp_expr(input, alloc)
            }),
            |expr| Expression::UnaryOp(UnaryOperator::Not(Not(alloc.alloc(expr)))),
        ),
        map(
            preceded(pair(tag("#"), lua_whitespace0), |input| {
                parse_exp_expr(input, alloc)
            }),
            |expr| Expression::UnaryOp(UnaryOperator::Length(Length(alloc.alloc(expr)))),
        ),
        map(
            preceded(pair(tag("-"), lua_whitespace0), |input| {
                parse_exp_expr(input, alloc)
            }),
            |expr| Expression::UnaryOp(UnaryOperator::Minus(Negation(alloc.alloc(expr)))),
        ),
        map(
            preceded(
                tuple((
                    tag("~"),
                    lua_whitespace0,
                    // This is required to disambiguate from not equals
                    not(tag("=")),
                )),
                |input| parse_exp_expr(input, alloc),
            ),
            |expr| Expression::UnaryOp(UnaryOperator::BitNot(BitNot(alloc.alloc(expr)))),
        ),
        |input| parse_exp_expr(input, alloc),
    ))(input)
}

pub fn parse_muldivmod_expr<'src, 'chunk>(
    input: Span<'src>,
    alloc: &'chunk ASTAllocator,
) -> ParseResult<'src, Expression<'chunk>> {
    parse_left_assoc_binop(
        input,
        |input| parse_unop_expr(input, alloc),
        |lhs, input| {
            if let (input, Some(rhs)) = opt(preceded(pair(tag("*"), lua_whitespace0), |input| {
                parse_unop_expr(input, alloc)
            }))(input)?
            {
                Ok((
                    input,
                    BinOpParseResult::Matched(BinaryOperator::Times(Times {
                        lhs: alloc.alloc(lhs),
                        rhs: alloc.alloc(rhs),
                    })),
                ))
            } else if let (input, Some(rhs)) =
                opt(preceded(pair(tag("//"), lua_whitespace0), |input| {
                    parse_unop_expr(input, alloc)
                }))(input)?
            {
                Ok((
                    input,
                    BinOpParseResult::Matched(BinaryOperator::IDiv(IDiv {
                        lhs: alloc.alloc(lhs),
                        rhs: alloc.alloc(rhs),
                    })),
                ))
            } else if let (input, Some(rhs)) =
                opt(preceded(pair(tag("/"), lua_whitespace0), |input| {
                    parse_unop_expr(input, alloc)
                }))(input)?
            {
                Ok((
                    input,
                    BinOpParseResult::Matched(BinaryOperator::Divide(Divide {
                        lhs: alloc.alloc(lhs),
                        rhs: alloc.alloc(rhs),
                    })),
                ))
            } else if let (input, Some(rhs)) =
                opt(preceded(pair(tag("%"), lua_whitespace0), |input| {
                    parse_unop_expr(input, alloc)
                }))(input)?
            {
                Ok((
                    input,
                    BinOpParseResult::Matched(BinaryOperator::Modulo(Modulo {
                        lhs: alloc.alloc(lhs),
                        rhs: alloc.alloc(rhs),
                    })),
                ))
            } else {
                Ok((input, BinOpParseResult::NoMatch(lhs)))
            }
        },
    )
}

pub fn parse_addsub_expr<'src, 'chunk>(
    input: Span<'src>,
    alloc: &'chunk ASTAllocator,
) -> ParseResult<'src, Expression<'chunk>> {
    parse_left_assoc_binop(
        input,
        |input| parse_muldivmod_expr(input, alloc),
        |lhs, input| {
            if let (input, Some(rhs)) = opt(preceded(pair(tag("+"), lua_whitespace0), |input| {
                parse_muldivmod_expr(input, alloc)
            }))(input)?
            {
                Ok((
                    input,
                    BinOpParseResult::Matched(BinaryOperator::Plus(Plus {
                        lhs: alloc.alloc(lhs),
                        rhs: alloc.alloc(rhs),
                    })),
                ))
            } else if let (input, Some(rhs)) =
                opt(preceded(pair(tag("-"), lua_whitespace0), |input| {
                    parse_muldivmod_expr(input, alloc)
                }))(input)?
            {
                Ok((
                    input,
                    BinOpParseResult::Matched(BinaryOperator::Minus(Minus {
                        lhs: alloc.alloc(lhs),
                        rhs: alloc.alloc(rhs),
                    })),
                ))
            } else {
                Ok((input, BinOpParseResult::NoMatch(lhs)))
            }
        },
    )
}

pub fn parse_concat_expr<'src, 'chunk>(
    input: Span<'src>,
    alloc: &'chunk ASTAllocator,
) -> ParseResult<'src, Expression<'chunk>> {
    parse_right_assoc_binop(
        input,
        |input| parse_addsub_expr(input, alloc),
        opt(preceded(pair(tag(".."), lua_whitespace0), |input| {
            parse_addsub_expr(input, alloc)
        })),
        |lhs, rhs| {
            BinaryOperator::Concat(Concat {
                lhs: alloc.alloc(lhs),
                rhs: alloc.alloc(rhs),
            })
        },
    )
}

pub fn parse_shift_expr<'src, 'chunk>(
    input: Span<'src>,
    alloc: &'chunk ASTAllocator,
) -> ParseResult<'src, Expression<'chunk>> {
    parse_left_assoc_binop(
        input,
        |input| parse_concat_expr(input, alloc),
        |lhs, input| {
            if let (input, Some(rhs)) = opt(preceded(pair(tag("<<"), lua_whitespace0), |input| {
                parse_concat_expr(input, alloc)
            }))(input)?
            {
                Ok((
                    input,
                    BinOpParseResult::Matched(BinaryOperator::ShiftLeft(ShiftLeft {
                        lhs: alloc.alloc(lhs),
                        rhs: alloc.alloc(rhs),
                    })),
                ))
            } else if let (input, Some(rhs)) =
                opt(preceded(pair(tag(">>"), lua_whitespace0), |input| {
                    parse_concat_expr(input, alloc)
                }))(input)?
            {
                Ok((
                    input,
                    BinOpParseResult::Matched(BinaryOperator::ShiftRight(ShiftRight {
                        lhs: alloc.alloc(lhs),
                        rhs: alloc.alloc(rhs),
                    })),
                ))
            } else {
                Ok((input, BinOpParseResult::NoMatch(lhs)))
            }
        },
    )
}

pub fn parse_bitand_expr<'src, 'chunk>(
    input: Span<'src>,
    alloc: &'chunk ASTAllocator,
) -> ParseResult<'src, Expression<'chunk>> {
    parse_left_assoc_binop(
        input,
        |input| parse_shift_expr(input, alloc),
        |lhs, input| {
            if let (input, Some(rhs)) = opt(preceded(pair(tag("&"), lua_whitespace0), |input| {
                parse_shift_expr(input, alloc)
            }))(input)?
            {
                Ok((
                    input,
                    BinOpParseResult::Matched(BinaryOperator::BitAnd(BitAnd {
                        lhs: alloc.alloc(lhs),
                        rhs: alloc.alloc(rhs),
                    })),
                ))
            } else {
                Ok((input, BinOpParseResult::NoMatch(lhs)))
            }
        },
    )
}

pub fn parse_bitxor_expr<'src, 'chunk>(
    input: Span<'src>,
    alloc: &'chunk ASTAllocator,
) -> ParseResult<'src, Expression<'chunk>> {
    parse_left_assoc_binop(
        input,
        |input| parse_bitand_expr(input, alloc),
        |lhs, input| {
            if let (input, Some(rhs)) = opt(preceded(
                tuple((tag("~"), lua_whitespace0, not(tag("=")))),
                |input| parse_bitand_expr(input, alloc),
            ))(input)?
            {
                Ok((
                    input,
                    BinOpParseResult::Matched(BinaryOperator::BitXor(BitXor {
                        lhs: alloc.alloc(lhs),
                        rhs: alloc.alloc(rhs),
                    })),
                ))
            } else {
                Ok((input, BinOpParseResult::NoMatch(lhs)))
            }
        },
    )
}

pub fn parse_bitor_expr<'src, 'chunk>(
    input: Span<'src>,
    alloc: &'chunk ASTAllocator,
) -> ParseResult<'src, Expression<'chunk>> {
    parse_left_assoc_binop(
        input,
        |input| parse_bitxor_expr(input, alloc),
        |lhs, input| {
            if let (input, Some(rhs)) = opt(preceded(pair(tag("|"), lua_whitespace0), |input| {
                parse_bitxor_expr(input, alloc)
            }))(input)?
            {
                Ok((
                    input,
                    BinOpParseResult::Matched(BinaryOperator::BitOr(BitOr {
                        lhs: alloc.alloc(lhs),
                        rhs: alloc.alloc(rhs),
                    })),
                ))
            } else {
                Ok((input, BinOpParseResult::NoMatch(lhs)))
            }
        },
    )
}

pub fn parse_logical_expr<'src, 'chunk>(
    input: Span<'src>,
    alloc: &'chunk ASTAllocator,
) -> ParseResult<'src, Expression<'chunk>> {
    parse_left_assoc_binop(
        input,
        |input| parse_bitor_expr(input, alloc),
        |lhs, input| {
            if let (input, Some(rhs)) = opt(preceded(pair(tag("<="), lua_whitespace0), |input| {
                parse_bitor_expr(input, alloc)
            }))(input)?
            {
                Ok((
                    input,
                    BinOpParseResult::Matched(BinaryOperator::LessEqual(LessEqual {
                        lhs: alloc.alloc(lhs),
                        rhs: alloc.alloc(rhs),
                    })),
                ))
            } else if let (input, Some(rhs)) =
                opt(preceded(pair(tag("<"), lua_whitespace0), |input| {
                    parse_bitor_expr(input, alloc)
                }))(input)?
            {
                Ok((
                    input,
                    BinOpParseResult::Matched(BinaryOperator::LessThan(LessThan {
                        lhs: alloc.alloc(lhs),
                        rhs: alloc.alloc(rhs),
                    })),
                ))
            } else if let (input, Some(rhs)) =
                opt(preceded(pair(tag(">="), lua_whitespace0), |input| {
                    parse_bitor_expr(input, alloc)
                }))(input)?
            {
                Ok((
                    input,
                    BinOpParseResult::Matched(BinaryOperator::GreaterEqual(GreaterEqual {
                        lhs: alloc.alloc(lhs),
                        rhs: alloc.alloc(rhs),
                    })),
                ))
            } else if let (input, Some(rhs)) =
                opt(preceded(pair(tag(">"), lua_whitespace0), |input| {
                    parse_bitor_expr(input, alloc)
                }))(input)?
            {
                Ok((
                    input,
                    BinOpParseResult::Matched(BinaryOperator::GreaterThan(GreaterThan {
                        lhs: alloc.alloc(lhs),
                        rhs: alloc.alloc(rhs),
                    })),
                ))
            } else if let (input, Some(rhs)) =
                opt(preceded(pair(tag("=="), lua_whitespace0), |input| {
                    parse_bitor_expr(input, alloc)
                }))(input)?
            {
                Ok((
                    input,
                    BinOpParseResult::Matched(BinaryOperator::Equals(Equals {
                        lhs: alloc.alloc(lhs),
                        rhs: alloc.alloc(rhs),
                    })),
                ))
            } else if let (input, Some(rhs)) =
                opt(preceded(pair(tag("~="), lua_whitespace0), |input| {
                    parse_bitor_expr(input, alloc)
                }))(input)?
            {
                Ok((
                    input,
                    BinOpParseResult::Matched(BinaryOperator::NotEqual(NotEqual {
                        lhs: alloc.alloc(lhs),
                        rhs: alloc.alloc(rhs),
                    })),
                ))
            } else {
                Ok((input, BinOpParseResult::NoMatch(lhs)))
            }
        },
    )
}

pub fn parse_and_expr<'src, 'chunk>(
    input: Span<'src>,
    alloc: &'chunk ASTAllocator,
) -> ParseResult<'src, Expression<'chunk>> {
    parse_left_assoc_binop(
        input,
        |input| parse_logical_expr(input, alloc),
        |lhs, input| {
            if let (input, Some(rhs)) = opt(preceded(
                pair(
                    tag("and"),
                    // This is required to disambiguate from an identifier
                    lua_whitespace1,
                ),
                |input| parse_logical_expr(input, alloc),
            ))(input)?
            {
                Ok((
                    input,
                    BinOpParseResult::Matched(BinaryOperator::And(And {
                        lhs: alloc.alloc(lhs),
                        rhs: alloc.alloc(rhs),
                    })),
                ))
            } else {
                Ok((input, BinOpParseResult::NoMatch(lhs)))
            }
        },
    )
}

pub fn parse_or_expr<'src, 'chunk>(
    input: Span<'src>,
    alloc: &'chunk ASTAllocator,
) -> ParseResult<'src, Expression<'chunk>> {
    parse_left_assoc_binop(
        input,
        |input| parse_and_expr(input, alloc),
        |lhs, input| {
            if let (input, Some(rhs)) = opt(preceded(
                pair(
                    tag("or"),
                    // This is required to disambiguate from an identifier
                    lua_whitespace1,
                ),
                |input| parse_and_expr(input, alloc),
            ))(input)?
            {
                Ok((
                    input,
                    BinOpParseResult::Matched(BinaryOperator::Or(Or {
                        lhs: alloc.alloc(lhs),
                        rhs: alloc.alloc(rhs),
                    })),
                ))
            } else {
                Ok((input, BinOpParseResult::NoMatch(lhs)))
            }
        },
    )
}

fn parse_left_assoc_binop<'src, 'chunk, F, N>(
    mut input: Span<'src>,
    mut first_expr: F,
    mut next_expr: N,
) -> ParseResult<'src, Expression<'chunk>>
where
    F: FnMut(Span<'src>) -> ParseResult<'src, Expression<'chunk>>,
    N: FnMut(Expression<'chunk>, Span<'src>) -> ParseResult<'src, BinOpParseResult<'chunk>>,
{
    let (remain, mut lhs) = first_expr(input)?;
    input = remain;

    loop {
        let (remain, ()) = lua_whitespace0(input)?;
        input = remain;

        let (remain, next) = next_expr(lhs, input)?;
        input = remain;

        match next {
            BinOpParseResult::NoMatch(expr) => return Ok((input, expr)),
            BinOpParseResult::Matched(next) => lhs = Expression::BinaryOp(next),
        }
    }
}

fn parse_right_assoc_binop<'src, 'chunk, F, N, C>(
    mut input: Span<'src>,
    mut first_expr: F,
    mut next_expr: N,
    mut combine_exprs: C,
) -> ParseResult<'src, Expression<'chunk>>
where
    F: FnMut(Span<'src>) -> ParseResult<'src, Expression<'chunk>>,
    N: FnMut(Span<'src>) -> ParseResult<'src, Option<Expression<'chunk>>>,
    C: FnMut(Expression<'chunk>, Expression<'chunk>) -> BinaryOperator<'chunk>,
{
    let (remain, lhs) = first_expr(input)?;
    input = remain;

    let mut exprs = vec![lhs];

    loop {
        let (remain, ()) = lua_whitespace0(input)?;
        input = remain;

        let (remain, rhs) = next_expr(input)?;
        input = remain;

        if let Some(rhs) = rhs {
            exprs.push(rhs);
        } else {
            break;
        }
    }

    let mut rhs = exprs
        .pop()
        .expect("at least one expression must be present");

    for lhs in exprs.into_iter().rev() {
        rhs = Expression::BinaryOp(combine_exprs(lhs, rhs));
    }

    Ok((input, rhs))
}
