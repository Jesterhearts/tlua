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
    ast::expressions::{
        operator::*,
        Expression,
    },
    parsing::{
        expressions::parse_non_op_expr,
        lua_whitespace0,
        lua_whitespace1,
        ASTAllocator,
        ParseResult,
        Span,
    },
};

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

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;

    use crate::{
        ast::{
            expressions::{
                operator::*,
                Expression,
                number::Number,
            },
            prefix_expression::VarPrefixExpression,
        },
            final_parser,
        parsing::{
            ASTAllocator,
            Parse,
            Span,
        },
    };

    #[test]
    pub fn parses_exponentiation() -> anyhow::Result<()> {
        let src = "1 ^ 2";

        let alloc = ASTAllocator::default();
        let expr =
            final_parser!(Span::new(src.as_bytes()) => |input| Expression::parse(input, &alloc))?;

        assert_eq!(
            expr,
            Expression::BinaryOp(BinaryOperator::Exponetiation(Exponetiation {
                lhs: &Expression::Number(Number::Integer(1)),
                rhs: &Expression::Number(Number::Integer(2)),
            }))
        );

        Ok(())
    }

    #[test]
    pub fn parses_unary_not() -> anyhow::Result<()> {
        let src = "not 1";

        let alloc = ASTAllocator::default();
        let expr =
            final_parser!(Span::new(src.as_bytes()) => |input| Expression::parse(input, &alloc))?;

        assert_eq!(
            expr,
            Expression::UnaryOp(UnaryOperator::Not(Not(&Expression::Number(
                Number::Integer(1)
            ))))
        );

        Ok(())
    }

    #[test]
    pub fn parses_unary_not_handles_ident() -> anyhow::Result<()> {
        let src = "not notabc";

        let alloc = ASTAllocator::default();
        let expr =
            final_parser!(Span::new(src.as_bytes()) => |input| Expression::parse(input, &alloc))?;

        assert_eq!(
            expr,
            Expression::UnaryOp(UnaryOperator::Not(Not(&Expression::Variable(
                &VarPrefixExpression::Name("notabc".into())
            ))))
        );

        Ok(())
    }

    #[test]
    pub fn parses_unary_len() -> anyhow::Result<()> {
        let src = "#1";

        let alloc = ASTAllocator::default();
        let expr =
            final_parser!(Span::new(src.as_bytes()) => |input| Expression::parse(input, &alloc))?;

        assert_eq!(
            expr,
            Expression::UnaryOp(UnaryOperator::Length(Length(&Expression::Number(
                Number::Integer(1)
            ))))
        );

        Ok(())
    }

    #[test]
    pub fn parses_unary_minus() -> anyhow::Result<()> {
        let src = "-1";

        let alloc = ASTAllocator::default();
        let expr =
            final_parser!(Span::new(src.as_bytes()) => |input| Expression::parse(input, &alloc))?;

        assert_eq!(
            expr,
            Expression::UnaryOp(UnaryOperator::Minus(Negation(&Expression::Number(
                Number::Integer(1)
            ))))
        );

        Ok(())
    }

    #[test]
    pub fn parses_unary_bitnot() -> anyhow::Result<()> {
        let src = "~1";

        let alloc = ASTAllocator::default();
        let expr =
            final_parser!(Span::new(src.as_bytes()) => |input| Expression::parse(input, &alloc))?;

        assert_eq!(
            expr,
            Expression::UnaryOp(UnaryOperator::BitNot(BitNot(&Expression::Number(
                Number::Integer(1)
            ))))
        );

        Ok(())
    }

    #[test]
    pub fn parses_times() -> anyhow::Result<()> {
        let src = "1 * 2";

        let alloc = ASTAllocator::default();
        let expr =
            final_parser!(Span::new(src.as_bytes()) => |input| Expression::parse(input, &alloc))?;

        assert_eq!(
            expr,
            Expression::BinaryOp(BinaryOperator::Times(Times {
                lhs: &Expression::Number(Number::Integer(1)),
                rhs: &Expression::Number(Number::Integer(2))
            }))
        );

        Ok(())
    }

    #[test]
    pub fn parses_div() -> anyhow::Result<()> {
        let src = "1 / 2";

        let alloc = ASTAllocator::default();
        let expr =
            final_parser!(Span::new(src.as_bytes()) => |input| Expression::parse(input, &alloc))?;

        assert_eq!(
            expr,
            Expression::BinaryOp(BinaryOperator::Divide(Divide {
                lhs: &Expression::Number(Number::Integer(1)),
                rhs: &Expression::Number(Number::Integer(2))
            }))
        );

        Ok(())
    }

    #[test]
    pub fn parses_idiv() -> anyhow::Result<()> {
        let src = "1 // 2";

        let alloc = ASTAllocator::default();
        let expr =
            final_parser!(Span::new(src.as_bytes()) => |input| Expression::parse(input, &alloc))?;

        assert_eq!(
            expr,
            Expression::BinaryOp(BinaryOperator::IDiv(IDiv {
                lhs: &Expression::Number(Number::Integer(1)),
                rhs: &Expression::Number(Number::Integer(2))
            }))
        );

        Ok(())
    }

    #[test]
    pub fn parses_modulo() -> anyhow::Result<()> {
        let src = "1 % 2";

        let alloc = ASTAllocator::default();
        let expr =
            final_parser!(Span::new(src.as_bytes()) => |input| Expression::parse(input, &alloc))?;

        assert_eq!(
            expr,
            Expression::BinaryOp(BinaryOperator::Modulo(Modulo {
                lhs: &Expression::Number(Number::Integer(1)),
                rhs: &Expression::Number(Number::Integer(2))
            }))
        );

        Ok(())
    }

    #[test]
    pub fn parses_add() -> anyhow::Result<()> {
        let src = "1 + 2";

        let alloc = ASTAllocator::default();
        let expr =
            final_parser!(Span::new(src.as_bytes()) => |input| Expression::parse(input, &alloc))?;

        assert_eq!(
            expr,
            Expression::BinaryOp(BinaryOperator::Plus(Plus {
                lhs: &Expression::Number(Number::Integer(1)),
                rhs: &Expression::Number(Number::Integer(2))
            }))
        );

        Ok(())
    }

    #[test]
    pub fn parses_minus() -> anyhow::Result<()> {
        let src = "1 - 2";

        let alloc = ASTAllocator::default();
        let expr =
            final_parser!(Span::new(src.as_bytes()) => |input| Expression::parse(input, &alloc))?;

        assert_eq!(
            expr,
            Expression::BinaryOp(BinaryOperator::Minus(Minus {
                lhs: &Expression::Number(Number::Integer(1)),
                rhs: &Expression::Number(Number::Integer(2))
            }))
        );

        Ok(())
    }

    #[test]
    pub fn parses_concat() -> anyhow::Result<()> {
        let src = "1 .. 2";

        let alloc = ASTAllocator::default();
        let expr =
            final_parser!(Span::new(src.as_bytes()) => |input| Expression::parse(input, &alloc))?;

        assert_eq!(
            expr,
            Expression::BinaryOp(BinaryOperator::Concat(Concat {
                lhs: &Expression::Number(Number::Integer(1)),
                rhs: &Expression::Number(Number::Integer(2))
            }))
        );

        Ok(())
    }

    #[test]
    pub fn parses_shiftl() -> anyhow::Result<()> {
        let src = "1 << 2";

        let alloc = ASTAllocator::default();
        let expr =
            final_parser!(Span::new(src.as_bytes()) => |input| Expression::parse(input, &alloc))?;

        assert_eq!(
            expr,
            Expression::BinaryOp(BinaryOperator::ShiftLeft(ShiftLeft {
                lhs: &Expression::Number(Number::Integer(1)),
                rhs: &Expression::Number(Number::Integer(2))
            }))
        );

        Ok(())
    }

    #[test]
    pub fn parses_shiftr() -> anyhow::Result<()> {
        let src = "1 >> 2";

        let alloc = ASTAllocator::default();
        let expr =
            final_parser!(Span::new(src.as_bytes()) => |input| Expression::parse(input, &alloc))?;

        assert_eq!(
            expr,
            Expression::BinaryOp(BinaryOperator::ShiftRight(ShiftRight {
                lhs: &Expression::Number(Number::Integer(1)),
                rhs: &Expression::Number(Number::Integer(2))
            }))
        );

        Ok(())
    }

    #[test]
    pub fn parses_bitand() -> anyhow::Result<()> {
        let src = "1 & 2";

        let alloc = ASTAllocator::default();
        let expr =
            final_parser!(Span::new(src.as_bytes()) => |input| Expression::parse(input, &alloc))?;

        assert_eq!(
            expr,
            Expression::BinaryOp(BinaryOperator::BitAnd(BitAnd {
                lhs: &Expression::Number(Number::Integer(1)),
                rhs: &Expression::Number(Number::Integer(2))
            }))
        );

        Ok(())
    }

    #[test]
    pub fn parses_bitxor() -> anyhow::Result<()> {
        let src = "1 ~ 2";

        let alloc = ASTAllocator::default();
        let expr =
            final_parser!(Span::new(src.as_bytes()) => |input| Expression::parse(input, &alloc))?;

        assert_eq!(
            expr,
            Expression::BinaryOp(BinaryOperator::BitXor(BitXor {
                lhs: &Expression::Number(Number::Integer(1)),
                rhs: &Expression::Number(Number::Integer(2))
            }))
        );

        Ok(())
    }

    #[test]
    pub fn parses_bitor() -> anyhow::Result<()> {
        let src = "1 | 2";

        let alloc = ASTAllocator::default();
        let expr =
            final_parser!(Span::new(src.as_bytes()) => |input| Expression::parse(input, &alloc))?;

        assert_eq!(
            expr,
            Expression::BinaryOp(BinaryOperator::BitOr(BitOr {
                lhs: &Expression::Number(Number::Integer(1)),
                rhs: &Expression::Number(Number::Integer(2))
            }))
        );

        Ok(())
    }

    #[test]
    pub fn parses_lt() -> anyhow::Result<()> {
        let src = "1 < 2";

        let alloc = ASTAllocator::default();
        let expr =
            final_parser!(Span::new(src.as_bytes()) => |input| Expression::parse(input, &alloc))?;

        assert_eq!(
            expr,
            Expression::BinaryOp(BinaryOperator::LessThan(LessThan {
                lhs: &Expression::Number(Number::Integer(1)),
                rhs: &Expression::Number(Number::Integer(2)),
            }))
        );

        Ok(())
    }

    #[test]
    pub fn parses_le() -> anyhow::Result<()> {
        let src = "1 <= 2";

        let alloc = ASTAllocator::default();
        let expr =
            final_parser!(Span::new(src.as_bytes()) => |input| Expression::parse(input, &alloc))?;

        assert_eq!(
            expr,
            Expression::BinaryOp(BinaryOperator::LessEqual(LessEqual {
                lhs: &Expression::Number(Number::Integer(1)),
                rhs: &Expression::Number(Number::Integer(2)),
            }))
        );

        Ok(())
    }

    #[test]
    pub fn parses_gt() -> anyhow::Result<()> {
        let src = "1 > 2";

        let alloc = ASTAllocator::default();
        let expr =
            final_parser!(Span::new(src.as_bytes()) => |input| Expression::parse(input, &alloc))?;

        assert_eq!(
            expr,
            Expression::BinaryOp(BinaryOperator::GreaterThan(GreaterThan {
                lhs: &Expression::Number(Number::Integer(1)),
                rhs: &Expression::Number(Number::Integer(2)),
            }))
        );

        Ok(())
    }

    #[test]
    pub fn parses_ge() -> anyhow::Result<()> {
        let src = "1 >= 2";

        let alloc = ASTAllocator::default();
        let expr =
            final_parser!(Span::new(src.as_bytes()) => |input| Expression::parse(input, &alloc))?;

        assert_eq!(
            expr,
            Expression::BinaryOp(BinaryOperator::GreaterEqual(GreaterEqual {
                lhs: &Expression::Number(Number::Integer(1)),
                rhs: &Expression::Number(Number::Integer(2)),
            }))
        );

        Ok(())
    }

    #[test]
    pub fn parses_ne() -> anyhow::Result<()> {
        let src = "1 ~= 2";

        let alloc = ASTAllocator::default();
        let expr =
            final_parser!(Span::new(src.as_bytes()) => |input| Expression::parse(input, &alloc))?;

        assert_eq!(
            expr,
            Expression::BinaryOp(BinaryOperator::NotEqual(NotEqual {
                lhs: &Expression::Number(Number::Integer(1)),
                rhs: &Expression::Number(Number::Integer(2)),
            }))
        );

        Ok(())
    }

    #[test]
    pub fn parses_eq() -> anyhow::Result<()> {
        let src = "1 == 2";

        let alloc = ASTAllocator::default();
        let expr =
            final_parser!(Span::new(src.as_bytes()) => |input| Expression::parse(input, &alloc))?;

        assert_eq!(
            expr,
            Expression::BinaryOp(BinaryOperator::Equals(Equals {
                lhs: &Expression::Number(Number::Integer(1)),
                rhs: &Expression::Number(Number::Integer(2)),
            }))
        );

        Ok(())
    }

    #[test]
    pub fn parses_and() -> anyhow::Result<()> {
        let src = "1 and 2";

        let alloc = ASTAllocator::default();
        let expr =
            final_parser!(Span::new(src.as_bytes()) => |input| Expression::parse(input, &alloc))?;

        assert_eq!(
            expr,
            Expression::BinaryOp(BinaryOperator::And(And {
                lhs: &Expression::Number(Number::Integer(1)),
                rhs: &Expression::Number(Number::Integer(2)),
            }))
        );

        Ok(())
    }

    #[test]
    pub fn parses_and_handles_ident() -> anyhow::Result<()> {
        let src = "1 and and2";

        let alloc = ASTAllocator::default();
        let expr =
            final_parser!(Span::new(src.as_bytes()) => |input| Expression::parse(input, &alloc))?;

        assert_eq!(
            expr,
            Expression::BinaryOp(BinaryOperator::And(And {
                lhs: &Expression::Number(Number::Integer(1)),
                rhs: &Expression::Variable(&VarPrefixExpression::Name("and2".into())),
            }))
        );

        Ok(())
    }

    #[test]
    pub fn parses_or() -> anyhow::Result<()> {
        let src = "1 or 2";

        let alloc = ASTAllocator::default();
        let expr =
            final_parser!(Span::new(src.as_bytes()) => |input| Expression::parse(input, &alloc))?;

        assert_eq!(
            expr,
            Expression::BinaryOp(BinaryOperator::Or(Or {
                lhs: &Expression::Number(Number::Integer(1)),
                rhs: &Expression::Number(Number::Integer(2)),
            }))
        );

        Ok(())
    }

    #[test]
    pub fn parses_or_handles_ident() -> anyhow::Result<()> {
        let src = "1 or or2";

        let alloc = ASTAllocator::default();
        let expr =
            final_parser!(Span::new(src.as_bytes()) => |input| Expression::parse(input, &alloc))?;

        assert_eq!(
            expr,
            Expression::BinaryOp(BinaryOperator::Or(Or {
                lhs: &Expression::Number(Number::Integer(1)),
                rhs: &Expression::Variable(&VarPrefixExpression::Name("or2".into())),
            }))
        );

        Ok(())
    }

    #[test]
    pub fn parses_multi_left_assoc() -> anyhow::Result<()> {
        let src = "1 == 2 <= 3 >= 4";

        let alloc = ASTAllocator::default();
        let expr =
            final_parser!(Span::new(src.as_bytes()) => |input| Expression::parse(input, &alloc))?;

        assert_eq!(
            expr,
            Expression::BinaryOp(BinaryOperator::GreaterEqual(GreaterEqual {
                lhs: &Expression::BinaryOp(BinaryOperator::LessEqual(LessEqual {
                    lhs: &Expression::BinaryOp(BinaryOperator::Equals(Equals {
                        lhs: &Expression::Number(Number::Integer(1)),
                        rhs: &Expression::Number(Number::Integer(2)),
                    })),
                    rhs: &Expression::Number(Number::Integer(3)),
                })),
                rhs: &Expression::Number(Number::Integer(4)),
            }))
        );

        Ok(())
    }

    #[test]
    pub fn parses_multi_right_assoc() -> anyhow::Result<()> {
        let src = "1 .. 2 .. 3 .. 4";

        let alloc = ASTAllocator::default();
        let expr =
            final_parser!(Span::new(src.as_bytes()) => |input| Expression::parse(input, &alloc))?;

        assert_eq!(
            expr,
            Expression::BinaryOp(BinaryOperator::Concat(Concat {
                lhs: &Expression::Number(Number::Integer(1)),
                rhs: &Expression::BinaryOp(BinaryOperator::Concat(Concat {
                    lhs: &Expression::Number(Number::Integer(2)),
                    rhs: &Expression::BinaryOp(BinaryOperator::Concat(Concat {
                        lhs: &Expression::Number(Number::Integer(3)),
                        rhs: &Expression::Number(Number::Integer(4))
                    }))
                }))
            }))
        );

        Ok(())
    }

    #[test]
    pub fn parses_multi_right_assoc_precedence() -> anyhow::Result<()> {
        let src = "1 .. 2 ^ 3 .. 4";

        let alloc = ASTAllocator::default();
        let expr =
            final_parser!(Span::new(src.as_bytes()) => |input| Expression::parse(input, &alloc))?;

        assert_eq!(
            expr,
            Expression::BinaryOp(BinaryOperator::Concat(Concat {
                lhs: &Expression::Number(Number::Integer(1)),
                rhs: &Expression::BinaryOp(BinaryOperator::Concat(Concat {
                    lhs: &Expression::BinaryOp(BinaryOperator::Exponetiation(Exponetiation {
                        lhs: &Expression::Number(Number::Integer(2)),
                        rhs: &Expression::Number(Number::Integer(3)),
                    })),
                    rhs: &Expression::Number(Number::Integer(4)),
                })),
            }))
        );

        Ok(())
    }

    #[test]
    pub fn parse_precedence_simple_down() -> anyhow::Result<()> {
        let src = "1 ^ -2 == 1 and true";
        let alloc = ASTAllocator::default();
        let expr =
            final_parser!(Span::new(src.as_bytes()) => |input| Expression::parse(input, &alloc))?;

        assert_eq!(
            expr,
            Expression::BinaryOp(BinaryOperator::And(And {
                lhs: &Expression::BinaryOp(BinaryOperator::Equals(Equals {
                    lhs: &Expression::BinaryOp(BinaryOperator::Exponetiation(Exponetiation {
                        lhs: &Expression::Number(Number::Integer(1)),
                        rhs: &Expression::Number(Number::Integer(-2)),
                    })),
                    rhs: &Expression::Number(Number::Integer(1)),
                })),
                rhs: &Expression::Bool(true)
            }))
        );

        Ok(())
    }

    #[test]
    pub fn parses_precedence_up() -> anyhow::Result<()> {
        let src = "1 or 2 and 3 < 4 | 5 ~ 6 & 7 << 8 .. 9 + 10 * -11 ^ 12";

        let alloc = ASTAllocator::default();
        let expr =
            final_parser!(Span::new(src.as_bytes()) => |input| Expression::parse(input, &alloc))?;

        assert_eq!(
            expr,
            Expression::BinaryOp ( BinaryOperator::Or(Or{
                lhs: &Expression::Number(Number::Integer(1)),
                rhs: &Expression::BinaryOp ( BinaryOperator::And(And{
                    lhs: &Expression::Number(Number::Integer(2)),
                    rhs: &Expression::BinaryOp(BinaryOperator::LessThan(LessThan{
                        lhs: &Expression::Number(Number::Integer(3)),
                        rhs: &Expression::BinaryOp(BinaryOperator::BitOr(BitOr{
                            lhs: &Expression::Number(Number::Integer(4)),
                            rhs: &Expression::BinaryOp(BinaryOperator::BitXor(BitXor{
                                lhs: &Expression::Number(Number::Integer(5)),
                                rhs: &Expression::BinaryOp(BinaryOperator::BitAnd(BitAnd{
                                    lhs: &Expression::Number(Number::Integer(6)),
                                    rhs: &Expression::BinaryOp(BinaryOperator::ShiftLeft(ShiftLeft{
                                        lhs: &Expression::Number(Number::Integer(7)),
                                        rhs: &Expression::BinaryOp(BinaryOperator::Concat(Concat{
                                            lhs: &Expression::Number(Number::Integer(8)),
                                            rhs: &Expression::BinaryOp(BinaryOperator::Plus(Plus{
                                                lhs: &Expression::Number(Number::Integer(9)),
                                                rhs: &Expression::BinaryOp(BinaryOperator::Times(Times{
                                                    lhs: &Expression::Number(Number::Integer(10)),
                                                    rhs: &Expression::UnaryOp(UnaryOperator::Minus(Negation(
                                                        &Expression::BinaryOp(
                                                            BinaryOperator::Exponetiation(Exponetiation{
                                                                lhs: &Expression::Number(Number::Integer(11)),
                                                                rhs: &Expression::Number(Number::Integer(12)),
                                                    })))))
                                                }))
                                            }))
                                        }))
                                    }))
                                }))
                            }))
                        }))
                    }))
                })),
            }))
        );

        Ok(())
    }

    #[test]
    pub fn parses_precedence_down() -> anyhow::Result<()> {
        let src = "1 ^ -2 / 3 - 4 .. 5 >> 6 & 7 ~ 8 | 9 > 10 and 11 or 12";

        let alloc = ASTAllocator::default();
        let expr =
            final_parser!(Span::new(src.as_bytes()) => |input| Expression::parse(input, &alloc))?;

        assert_eq!(
            expr,
            Expression::BinaryOp(BinaryOperator::Or(Or{
                lhs: &Expression::BinaryOp(BinaryOperator::And(And{
                    lhs: &Expression::BinaryOp(BinaryOperator::GreaterThan(GreaterThan{
                        lhs: &Expression::BinaryOp(BinaryOperator::BitOr(BitOr{
                            lhs: &Expression::BinaryOp(BinaryOperator::BitXor(BitXor{
                                lhs: &Expression::BinaryOp(BinaryOperator::BitAnd(BitAnd{
                                    lhs: &Expression::BinaryOp(BinaryOperator::ShiftRight(ShiftRight{
                                        lhs: &Expression::BinaryOp(BinaryOperator::Concat(Concat{
                                            lhs: &Expression::BinaryOp(BinaryOperator::Minus(Minus{
                                                lhs: &Expression::BinaryOp(BinaryOperator::Divide(Divide{
                                                    lhs: &Expression::BinaryOp(BinaryOperator::Exponetiation(Exponetiation{
                                                        lhs: &Expression::Number(Number::Integer(
                                                            1
                                                        )),
                                                        rhs: &Expression::Number(Number::Integer(
                                                            -2
                                                        )),
                                                    })),
                                                    rhs: &Expression::Number(Number::Integer(3)),
                                                })),
                                                rhs: &Expression::Number(Number::Integer(4)),
                                            })),
                                            rhs: &Expression::Number(Number::Integer(5)),
                                        })),
                                        rhs: &Expression::Number(Number::Integer(6)),
                                    })),
                                    rhs: &Expression::Number(Number::Integer(7)),
                                })),
                                rhs: &Expression::Number(Number::Integer(8)),
                            })),
                            rhs: &Expression::Number(Number::Integer(9)),
                        })),
                        rhs: &Expression::Number(Number::Integer(10)),
                    })),
                    rhs: &Expression::Number(Number::Integer(11)),
                })),
                rhs: &Expression::Number(Number::Integer(12)),
            }))
        );

        Ok(())
    }
}
