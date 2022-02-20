use pretty_assertions::assert_eq;

use crate::{
    expressions::{
        number::Number,
        operator::*,
        Expression,
    },
    final_parser,
    prefix_expression::VarPrefixExpression,
    ASTAllocator,
    Span,
};

#[test]
pub fn parses_exponentiation() -> anyhow::Result<()> {
    let src = "1 ^ 2";

    let alloc = ASTAllocator::default();
    let expr = final_parser!(Span::new(src.as_bytes()) => Expression::parser(&alloc))?;

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
    let expr = final_parser!(Span::new(src.as_bytes()) => Expression::parser(&alloc))?;

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
    let expr = final_parser!(Span::new(src.as_bytes()) => Expression::parser(&alloc))?;

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
    let expr = final_parser!(Span::new(src.as_bytes()) => Expression::parser(&alloc))?;

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
    let expr = final_parser!(Span::new(src.as_bytes()) => Expression::parser(&alloc))?;

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
    let expr = final_parser!(Span::new(src.as_bytes()) => Expression::parser(&alloc))?;

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
    let expr = final_parser!(Span::new(src.as_bytes()) => Expression::parser(&alloc))?;

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
    let expr = final_parser!(Span::new(src.as_bytes()) => Expression::parser(&alloc))?;

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
    let expr = final_parser!(Span::new(src.as_bytes()) => Expression::parser(&alloc))?;

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
    let expr = final_parser!(Span::new(src.as_bytes()) => Expression::parser(&alloc))?;

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
    let expr = final_parser!(Span::new(src.as_bytes()) => Expression::parser(&alloc))?;

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
    let expr = final_parser!(Span::new(src.as_bytes()) => Expression::parser(&alloc))?;

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
    let expr = final_parser!(Span::new(src.as_bytes()) => Expression::parser(&alloc))?;

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
    let expr = final_parser!(Span::new(src.as_bytes()) => Expression::parser(&alloc))?;

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
    let expr = final_parser!(Span::new(src.as_bytes()) => Expression::parser(&alloc))?;

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
    let expr = final_parser!(Span::new(src.as_bytes()) => Expression::parser(&alloc))?;

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
    let expr = final_parser!(Span::new(src.as_bytes()) => Expression::parser(&alloc))?;

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
    let expr = final_parser!(Span::new(src.as_bytes()) => Expression::parser(&alloc))?;

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
    let expr = final_parser!(Span::new(src.as_bytes()) => Expression::parser(&alloc))?;

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
    let expr = final_parser!(Span::new(src.as_bytes()) => Expression::parser(&alloc))?;

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
    let expr = final_parser!(Span::new(src.as_bytes()) => Expression::parser(&alloc))?;

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
    let expr = final_parser!(Span::new(src.as_bytes()) => Expression::parser(&alloc))?;

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
    let expr = final_parser!(Span::new(src.as_bytes()) => Expression::parser(&alloc))?;

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
    let expr = final_parser!(Span::new(src.as_bytes()) => Expression::parser(&alloc))?;

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
    let expr = final_parser!(Span::new(src.as_bytes()) => Expression::parser(&alloc))?;

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
    let expr = final_parser!(Span::new(src.as_bytes()) => Expression::parser(&alloc))?;

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
    let expr = final_parser!(Span::new(src.as_bytes()) => Expression::parser(&alloc))?;

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
    let expr = final_parser!(Span::new(src.as_bytes()) => Expression::parser(&alloc))?;

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
    let expr = final_parser!(Span::new(src.as_bytes()) => Expression::parser(&alloc))?;

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
    let expr = final_parser!(Span::new(src.as_bytes()) => Expression::parser(&alloc))?;

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
    let expr = final_parser!(Span::new(src.as_bytes()) => Expression::parser(&alloc))?;

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
    let expr = final_parser!(Span::new(src.as_bytes()) => Expression::parser(&alloc))?;

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
    let expr = final_parser!(Span::new(src.as_bytes()) => Expression::parser(&alloc))?;

    assert_eq!(
        expr,
        Expression::BinaryOp(BinaryOperator::Or(Or {
            lhs: &Expression::Number(Number::Integer(1)),
            rhs: &Expression::BinaryOp(BinaryOperator::And(And {
                lhs: &Expression::Number(Number::Integer(2)),
                rhs: &Expression::BinaryOp(BinaryOperator::LessThan(LessThan {
                    lhs: &Expression::Number(Number::Integer(3)),
                    rhs: &Expression::BinaryOp(BinaryOperator::BitOr(BitOr {
                        lhs: &Expression::Number(Number::Integer(4)),
                        rhs: &Expression::BinaryOp(BinaryOperator::BitXor(BitXor {
                            lhs: &Expression::Number(Number::Integer(5)),
                            rhs: &Expression::BinaryOp(BinaryOperator::BitAnd(BitAnd {
                                lhs: &Expression::Number(Number::Integer(6)),
                                rhs: &Expression::BinaryOp(BinaryOperator::ShiftLeft(ShiftLeft {
                                    lhs: &Expression::Number(Number::Integer(7)),
                                    rhs: &Expression::BinaryOp(BinaryOperator::Concat(Concat {
                                        lhs: &Expression::Number(Number::Integer(8)),
                                        rhs: &Expression::BinaryOp(BinaryOperator::Plus(Plus {
                                            lhs: &Expression::Number(Number::Integer(9)),
                                            rhs: &Expression::BinaryOp(BinaryOperator::Times(
                                                Times {
                                                    lhs: &Expression::Number(Number::Integer(10)),
                                                    rhs: &Expression::UnaryOp(
                                                        UnaryOperator::Minus(Negation(
                                                            &Expression::BinaryOp(
                                                                BinaryOperator::Exponetiation(
                                                                    Exponetiation {
                                                                        lhs: &Expression::Number(
                                                                            Number::Integer(11)
                                                                        ),
                                                                        rhs: &Expression::Number(
                                                                            Number::Integer(12)
                                                                        ),
                                                                    }
                                                                )
                                                            )
                                                        ))
                                                    )
                                                }
                                            ))
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
    let expr = final_parser!(Span::new(src.as_bytes()) => Expression::parser(&alloc))?;

    assert_eq!(
        expr,
        Expression::BinaryOp(BinaryOperator::Or(Or {
            lhs: &Expression::BinaryOp(BinaryOperator::And(And {
                lhs: &Expression::BinaryOp(BinaryOperator::GreaterThan(GreaterThan {
                    lhs: &Expression::BinaryOp(BinaryOperator::BitOr(BitOr {
                        lhs: &Expression::BinaryOp(BinaryOperator::BitXor(BitXor {
                            lhs: &Expression::BinaryOp(BinaryOperator::BitAnd(BitAnd {
                                lhs: &Expression::BinaryOp(BinaryOperator::ShiftRight(
                                    ShiftRight {
                                        lhs: &Expression::BinaryOp(BinaryOperator::Concat(
                                            Concat {
                                                lhs: &Expression::BinaryOp(BinaryOperator::Minus(
                                                    Minus {
                                                        lhs: &Expression::BinaryOp(
                                                            BinaryOperator::Divide(Divide {
                                                                lhs: &Expression::BinaryOp(
                                                                    BinaryOperator::Exponetiation(
                                                                        Exponetiation {
                                                                            lhs:
                                                                                &Expression::Number(
                                                                                    Number::Integer(
                                                                                        1
                                                                                    )
                                                                                ),
                                                                            rhs:
                                                                                &Expression::Number(
                                                                                    Number::Integer(
                                                                                        -2
                                                                                    )
                                                                                ),
                                                                        }
                                                                    )
                                                                ),
                                                                rhs: &Expression::Number(
                                                                    Number::Integer(3)
                                                                ),
                                                            })
                                                        ),
                                                        rhs: &Expression::Number(Number::Integer(
                                                            4
                                                        )),
                                                    }
                                                )),
                                                rhs: &Expression::Number(Number::Integer(5)),
                                            }
                                        )),
                                        rhs: &Expression::Number(Number::Integer(6)),
                                    }
                                )),
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
