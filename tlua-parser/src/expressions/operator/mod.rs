use crate::{
    expressions::Expression,
    lexer::{
        SpannedToken,
        Token,
    },
    token_subset,
    ASTAllocator,
    ParseError,
    PeekableLexer,
    SyntaxError,
};

#[cfg(test)]
mod tests;

#[derive(Debug, PartialEq)]
pub enum UnaryOperator<'chunk> {
    Minus(Negation<'chunk>),
    Not(Not<'chunk>),
    Length(Length<'chunk>),
    BitNot(BitNot<'chunk>),
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
pub struct Exponetiation<'chunk> {
    pub lhs: &'chunk Expression<'chunk>,
    pub rhs: &'chunk Expression<'chunk>,
}

impl<'chunk> Exponetiation<'chunk> {
    /// This is the top of our precedence tree.
    /// We first try to match a non-operator expression since we know that would
    /// be a leaf. If we attempted to match any expression we would recurse
    /// infinitely. Since we reach this node attempting to parse a unary
    /// operator, we don't have to worry about handling that case.
    /// If we aren't looking at an exponentation expression, we just return
    /// whatever leaf we found.
    pub(crate) fn parse(
        lexer: &mut PeekableLexer,
        alloc: &'chunk ASTAllocator,
    ) -> Result<Option<Expression<'chunk>>, ParseError> {
        parse_right_assoc_binop(
            lexer,
            alloc,
            Expression::parse_leaf,
            parse_unary,
            Token::Caret,
            |lhs, rhs| {
                Expression::BinaryOp(BinaryOperator::Exponetiation(Exponetiation { lhs, rhs }))
            },
        )
    }
}

#[derive(Debug, PartialEq)]
pub struct Negation<'chunk>(pub &'chunk Expression<'chunk>);

#[derive(Debug, PartialEq)]
pub struct Not<'chunk>(pub &'chunk Expression<'chunk>);

#[derive(Debug, PartialEq)]
pub struct Length<'chunk>(pub &'chunk Expression<'chunk>);

#[derive(Debug, PartialEq)]
pub struct BitNot<'chunk>(pub &'chunk Expression<'chunk>);

pub(crate) fn parse_unary<'chunk>(
    lexer: &mut PeekableLexer,
    alloc: &'chunk ASTAllocator,
) -> Result<Option<Expression<'chunk>>, ParseError> {
    token_subset! {
        UnaryToken {
            Token::KWnot,
            Token::Hashtag,
            Token::Minus,
            Token::Tilde,
            Error(SyntaxError::ExpectedExpression)
        }
    }

    let token = if let Some(token) = UnaryToken::next(lexer) {
        token
    } else {
        return Exponetiation::parse(lexer, alloc);
    };

    Ok(
        Exponetiation::parse(lexer, alloc)?.map(|expr| match token.as_ref() {
            UnaryToken::KWnot => Expression::UnaryOp(UnaryOperator::Not(Not(alloc.alloc(expr)))),
            UnaryToken::Hashtag => {
                Expression::UnaryOp(UnaryOperator::Length(Length(alloc.alloc(expr))))
            }
            UnaryToken::Minus => {
                Expression::UnaryOp(UnaryOperator::Minus(Negation(alloc.alloc(expr))))
            }
            UnaryToken::Tilde => {
                Expression::UnaryOp(UnaryOperator::BitNot(BitNot(alloc.alloc(expr))))
            }
        }),
    )
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

pub(crate) fn parse_muldivmod<'chunk>(
    lexer: &mut PeekableLexer,
    alloc: &'chunk ASTAllocator,
) -> Result<Option<Expression<'chunk>>, ParseError> {
    parse_left_assoc_binop(
        lexer,
        alloc,
        parse_unary,
        |tok| {
            matches!(
                tok.as_ref(),
                Token::Star | Token::Slash | Token::DoubleSlash | Token::Percent
            )
        },
        |token, lhs, rhs| match token {
            Token::Star => Expression::BinaryOp(BinaryOperator::Times(Times { lhs, rhs })),
            Token::Slash => Expression::BinaryOp(BinaryOperator::Divide(Divide { lhs, rhs })),
            Token::DoubleSlash => Expression::BinaryOp(BinaryOperator::IDiv(IDiv { lhs, rhs })),
            Token::Percent => Expression::BinaryOp(BinaryOperator::Modulo(Modulo { lhs, rhs })),
            _ => unreachable!(),
        },
    )
}

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

pub(crate) fn parse_addsub<'chunk>(
    lexer: &mut PeekableLexer,
    alloc: &'chunk ASTAllocator,
) -> Result<Option<Expression<'chunk>>, ParseError> {
    parse_left_assoc_binop(
        lexer,
        alloc,
        parse_muldivmod,
        |tok| matches!(tok.as_ref(), Token::Minus | Token::Plus),
        |token, lhs, rhs| match token {
            Token::Minus => Expression::BinaryOp(BinaryOperator::Minus(Minus { lhs, rhs })),
            Token::Plus => Expression::BinaryOp(BinaryOperator::Plus(Plus { lhs, rhs })),
            _ => unreachable!(),
        },
    )
}

#[derive(Debug, PartialEq)]
pub struct Concat<'chunk> {
    pub lhs: &'chunk Expression<'chunk>,
    pub rhs: &'chunk Expression<'chunk>,
}

impl<'chunk> Concat<'chunk> {
    pub(crate) fn parse(
        lexer: &mut PeekableLexer,
        alloc: &'chunk ASTAllocator,
    ) -> Result<Option<Expression<'chunk>>, ParseError> {
        parse_right_assoc_binop(
            lexer,
            alloc,
            parse_addsub,
            Self::parse,
            Token::DoublePeriod,
            |lhs, rhs| Expression::BinaryOp(BinaryOperator::Concat(Self { lhs, rhs })),
        )
    }
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

pub(crate) fn parse_shift<'chunk>(
    lexer: &mut PeekableLexer,
    alloc: &'chunk ASTAllocator,
) -> Result<Option<Expression<'chunk>>, ParseError> {
    parse_left_assoc_binop(
        lexer,
        alloc,
        Concat::parse,
        |tok| {
            matches!(
                tok.as_ref(),
                Token::DoubleLeftAngle | Token::DoubleRightAngle
            )
        },
        |token, lhs, rhs| match token {
            Token::DoubleLeftAngle => {
                Expression::BinaryOp(BinaryOperator::ShiftLeft(ShiftLeft { lhs, rhs }))
            }
            Token::DoubleRightAngle => {
                Expression::BinaryOp(BinaryOperator::ShiftRight(ShiftRight { lhs, rhs }))
            }
            _ => unreachable!(),
        },
    )
}

#[derive(Debug, PartialEq)]
pub struct BitAnd<'chunk> {
    pub lhs: &'chunk Expression<'chunk>,
    pub rhs: &'chunk Expression<'chunk>,
}

impl<'chunk> BitAnd<'chunk> {
    pub(crate) fn parse(
        lexer: &mut PeekableLexer,
        alloc: &'chunk ASTAllocator,
    ) -> Result<Option<Expression<'chunk>>, ParseError> {
        parse_left_assoc_binop(
            lexer,
            alloc,
            parse_shift,
            |tok| *tok == Token::Ampersand,
            |_, lhs, rhs| Expression::BinaryOp(BinaryOperator::BitAnd(Self { lhs, rhs })),
        )
    }
}

#[derive(Debug, PartialEq)]
pub struct BitXor<'chunk> {
    pub lhs: &'chunk Expression<'chunk>,
    pub rhs: &'chunk Expression<'chunk>,
}

impl<'chunk> BitXor<'chunk> {
    pub(crate) fn parse(
        lexer: &mut PeekableLexer,
        alloc: &'chunk ASTAllocator,
    ) -> Result<Option<Expression<'chunk>>, ParseError> {
        parse_left_assoc_binop(
            lexer,
            alloc,
            BitAnd::parse,
            |tok| *tok == Token::Tilde,
            |_, lhs, rhs| Expression::BinaryOp(BinaryOperator::BitXor(Self { lhs, rhs })),
        )
    }
}

#[derive(Debug, PartialEq)]
pub struct BitOr<'chunk> {
    pub lhs: &'chunk Expression<'chunk>,
    pub rhs: &'chunk Expression<'chunk>,
}

impl<'chunk> BitOr<'chunk> {
    pub(crate) fn parse(
        lexer: &mut PeekableLexer,
        alloc: &'chunk ASTAllocator,
    ) -> Result<Option<Expression<'chunk>>, ParseError> {
        parse_left_assoc_binop(
            lexer,
            alloc,
            BitXor::parse,
            |tok| *tok == Token::Pipe,
            |_, lhs, rhs| Expression::BinaryOp(BinaryOperator::BitOr(Self { lhs, rhs })),
        )
    }
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

pub(crate) fn parse_logical<'chunk>(
    lexer: &mut PeekableLexer,
    alloc: &'chunk ASTAllocator,
) -> Result<Option<Expression<'chunk>>, ParseError> {
    parse_left_assoc_binop(
        lexer,
        alloc,
        BitOr::parse,
        |tok| {
            matches!(
                tok.as_ref(),
                Token::LeftAngle
                    | Token::RightAngle
                    | Token::LeftAngleEquals
                    | Token::RightAngleEquals
                    | Token::TildeEquals
                    | Token::DoubleEquals,
            )
        },
        |token, lhs, rhs| match token {
            Token::LeftAngle => {
                Expression::BinaryOp(BinaryOperator::LessThan(LessThan { lhs, rhs }))
            }
            Token::RightAngle => {
                Expression::BinaryOp(BinaryOperator::GreaterThan(GreaterThan { lhs, rhs }))
            }
            Token::LeftAngleEquals => {
                Expression::BinaryOp(BinaryOperator::LessEqual(LessEqual { lhs, rhs }))
            }
            Token::RightAngleEquals => {
                Expression::BinaryOp(BinaryOperator::GreaterEqual(GreaterEqual { lhs, rhs }))
            }
            Token::TildeEquals => {
                Expression::BinaryOp(BinaryOperator::NotEqual(NotEqual { lhs, rhs }))
            }
            Token::DoubleEquals => {
                Expression::BinaryOp(BinaryOperator::Equals(Equals { lhs, rhs }))
            }
            _ => unreachable!(),
        },
    )
}

#[derive(Debug, PartialEq)]
pub struct And<'chunk> {
    pub lhs: &'chunk Expression<'chunk>,
    pub rhs: &'chunk Expression<'chunk>,
}

impl<'chunk> And<'chunk> {
    pub(crate) fn parse(
        lexer: &mut PeekableLexer,
        alloc: &'chunk ASTAllocator,
    ) -> Result<Option<Expression<'chunk>>, ParseError> {
        parse_left_assoc_binop(
            lexer,
            alloc,
            parse_logical,
            |tok| *tok == Token::KWand,
            |_, lhs, rhs| Expression::BinaryOp(BinaryOperator::And(Self { lhs, rhs })),
        )
    }
}

#[derive(Debug, PartialEq)]
pub struct Or<'chunk> {
    pub lhs: &'chunk Expression<'chunk>,
    pub rhs: &'chunk Expression<'chunk>,
}

impl<'chunk> Or<'chunk> {
    pub(crate) fn parse(
        lexer: &mut PeekableLexer,
        alloc: &'chunk ASTAllocator,
    ) -> Result<Option<Expression<'chunk>>, ParseError> {
        parse_left_assoc_binop(
            lexer,
            alloc,
            And::parse,
            |tok| *tok == Token::KWor,
            |_, lhs, rhs| Expression::BinaryOp(BinaryOperator::Or(Self { lhs, rhs })),
        )
    }
}

fn parse_left_assoc_binop<'src, 'chunk, F, M, C>(
    lexer: &mut PeekableLexer,
    alloc: &'chunk ASTAllocator,
    higher_precedent: F,
    match_next: M,
    combine: C,
) -> Result<Option<Expression<'chunk>>, ParseError>
where
    F: Fn(
        &mut PeekableLexer,
        &'chunk ASTAllocator,
    ) -> Result<Option<Expression<'chunk>>, ParseError>,
    M: Fn(&SpannedToken) -> bool,
    C: Fn(Token, &'chunk Expression<'chunk>, &'chunk Expression<'chunk>) -> Expression<'chunk>,
{
    let mut lhs = if let Some(lhs) = higher_precedent(lexer, alloc)? {
        lhs
    } else {
        return Ok(None);
    };

    loop {
        let token = if let Some(token) = lexer.next_if(&match_next) {
            token
        } else {
            return Ok(Some(lhs));
        };

        let rhs = higher_precedent(lexer, alloc)?
            .ok_or_else(|| ParseError::from_here(lexer, SyntaxError::ExpectedExpression))?;

        lhs = combine(token.token, &*alloc.alloc(lhs), &*alloc.alloc(rhs));
    }
}

fn parse_right_assoc_binop<'src, 'chunk, F1, F2, C>(
    lexer: &mut PeekableLexer,
    alloc: &'chunk ASTAllocator,
    higher_precedent: F1,
    equal_precedent: F2,
    match_next: Token,
    combine: C,
) -> Result<Option<Expression<'chunk>>, ParseError>
where
    F1: Fn(
        &mut PeekableLexer,
        &'chunk ASTAllocator,
    ) -> Result<Option<Expression<'chunk>>, ParseError>,
    F2: Fn(
        &mut PeekableLexer,
        &'chunk ASTAllocator,
    ) -> Result<Option<Expression<'chunk>>, ParseError>,
    C: Fn(&'chunk Expression<'chunk>, &'chunk Expression<'chunk>) -> Expression<'chunk>,
{
    let lhs = if let Some(lhs) = higher_precedent(lexer, alloc)? {
        lhs
    } else {
        return Ok(None);
    };

    let mut exprs = vec![lhs];
    while lexer.next_if_eq(match_next).is_some() {
        let rhs = if let Some(rhs) = higher_precedent(lexer, alloc)? {
            rhs
        } else if let Some(rhs) = equal_precedent(lexer, alloc)? {
            rhs
        } else {
            break;
        };

        exprs.push(rhs);
    }

    let mut rhs = exprs.pop().expect("At least one expression");
    for lhs in exprs.into_iter().rev() {
        rhs = combine(&*alloc.alloc(lhs), &*alloc.alloc(rhs));
    }

    Ok(Some(rhs))
}
