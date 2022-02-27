use crate::{
    expressions::Expression,
    identifiers::Ident,
    lexer::Token,
    list::List,
    parse_list1_split_tail,
    ASTAllocator,
    ParseError,
    ParseErrorExt,
    PeekableLexer,
    SyntaxError,
};

pub mod function_calls;
use self::function_calls::FnArgs;

#[derive(Debug, PartialEq)]
pub enum HeadAtom<'chunk> {
    Name(Ident),
    Parenthesized(&'chunk Expression<'chunk>),
}

impl<'chunk> HeadAtom<'chunk> {
    fn parse(lexer: &mut PeekableLexer, alloc: &'chunk ASTAllocator) -> Result<Self, ParseError> {
        Ident::parse(lexer, alloc).map(Self::Name).recover_with(|| {
            lexer.next_if_eq(Token::LParen).ok_or_else(|| {
                ParseError::recoverable_from_here(lexer, SyntaxError::ExpectedToken(Token::LParen))
            })?;

            Expression::parse(lexer, alloc)
                .mark_unrecoverable()
                .and_then(|expr| {
                    lexer.next_if_eq(Token::RParen).ok_or_else(|| {
                        ParseError::unrecoverable_from_here(
                            lexer,
                            SyntaxError::ExpectedToken(Token::RParen),
                        )
                    })?;
                    Ok(Self::Parenthesized(alloc.alloc(expr)))
                })
        })
    }
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

impl<'chunk> PrefixAtom<'chunk> {
    fn parse(lexer: &mut PeekableLexer, alloc: &'chunk ASTAllocator) -> Result<Self, ParseError> {
        lexer
            .next_if_eq(Token::LBracket)
            .ok_or_else(|| {
                ParseError::recoverable_from_here(
                    lexer,
                    SyntaxError::ExpectedToken(Token::LBracket),
                )
            })
            .and_then(|_| {
                Expression::parse(lexer, alloc)
                    .mark_unrecoverable()
                    .and_then(|expr| {
                        lexer.next_if_eq(Token::RBracket).ok_or_else(|| {
                            ParseError::unrecoverable_from_here(
                                lexer,
                                SyntaxError::ExpectedToken(Token::RBracket),
                            )
                        })?;
                        Ok(Self::Var(VarAtom::IndexOp(expr)))
                    })
            })
            .recover_with(|| {
                lexer.next_if_eq(Token::Period).ok_or_else(|| {
                    ParseError::recoverable_from_here(
                        lexer,
                        SyntaxError::ExpectedToken(Token::Period),
                    )
                })?;

                Ident::parse(lexer, alloc)
                    .mark_unrecoverable()
                    .map(|ident| Self::Var(VarAtom::Name(ident)))
            })
            .recover_with(|| {
                lexer.next_if_eq(Token::Colon).ok_or_else(|| {
                    ParseError::recoverable_from_here(
                        lexer,
                        SyntaxError::ExpectedToken(Token::Colon),
                    )
                })?;

                let name = Ident::parse(lexer, alloc).mark_unrecoverable()?;
                FnArgs::parse(lexer, alloc)
                    .mark_unrecoverable()
                    .map(|args| Self::Function(FunctionAtom::MethodCall { name, args }))
            })
            .recover_with(|| {
                FnArgs::parse(lexer, alloc).map(|args| Self::Function(FunctionAtom::Call(args)))
            })
            .ok_or_else(|| {
                ParseError::recoverable_from_here(lexer, SyntaxError::ExpectedPrefixExpression)
            })
    }
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

impl<'chunk> PrefixExpression<'chunk> {
    pub(crate) fn parse(
        lexer: &mut PeekableLexer,
        alloc: &'chunk ASTAllocator,
    ) -> Result<Self, ParseError> {
        let head = HeadAtom::parse(lexer, alloc)?;
        let rest = parse_list1_split_tail(lexer, alloc, PrefixAtom::parse).recover()?;

        if let Some((middle, tail)) = rest {
            Ok(match tail {
                PrefixAtom::Var(var) => Self::Variable(VarPrefixExpression::TableAccess {
                    head,
                    middle,
                    last: alloc.alloc(var),
                }),
                PrefixAtom::Function(args) => {
                    if middle.is_empty() {
                        Self::FnCall(FnCallPrefixExpression::Call { head, args })
                    } else {
                        Self::FnCall(FnCallPrefixExpression::CallPath {
                            head,
                            middle,
                            last: args,
                        })
                    }
                }
            })
        } else {
            Ok(match head {
                HeadAtom::Name(name) => Self::Variable(VarPrefixExpression::Name(name)),
                HeadAtom::Parenthesized(expr) => Self::Parenthesized(expr),
            })
        }
    }
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;

    use super::PrefixExpression;
    use crate::{
        expressions::{
            strings::ConstantString,
            tables::TableConstructor,
            Expression,
        },
        final_parser,
        identifiers::Ident,
        list::{
            List,
            ListNode,
        },
        prefix_expression::{
            function_calls::FnArgs,
            FnCallPrefixExpression,
            FunctionAtom,
            HeadAtom,
            PrefixAtom,
            VarAtom,
            VarPrefixExpression,
        },
        ASTAllocator,
        StringTable,
    };

    #[test]
    pub fn parses_dotted_path() -> anyhow::Result<()> {
        let src = "a.b.c";

        let alloc = ASTAllocator::default();
        let mut strings = StringTable::default();
        let result =
            final_parser!((src.as_bytes(), &alloc, &mut strings) => PrefixExpression::parse)?;

        assert_eq!(
            result,
            PrefixExpression::Variable(VarPrefixExpression::TableAccess {
                head: HeadAtom::Name(Ident(0)),
                middle: List::new(&mut ListNode::new(PrefixAtom::Var(VarAtom::Name(Ident(1))))),
                last: &VarAtom::Name(Ident(2))
            })
        );

        Ok(())
    }

    #[test]
    pub fn parses_bracket_path() -> anyhow::Result<()> {
        let src = "a[b][c]";

        let alloc = ASTAllocator::default();
        let mut strings = StringTable::default();
        let result =
            final_parser!((src.as_bytes(), &alloc, &mut strings) => PrefixExpression::parse)?;

        assert_eq!(
            result,
            PrefixExpression::Variable(VarPrefixExpression::TableAccess {
                head: HeadAtom::Name(Ident(0)),
                middle: List::new(&mut ListNode::new(PrefixAtom::Var(VarAtom::IndexOp(
                    Expression::Variable(&VarPrefixExpression::Name(Ident(1)))
                ))),),
                last: &VarAtom::IndexOp(Expression::Variable(&VarPrefixExpression::Name(Ident(2)))),
            })
        );

        Ok(())
    }

    #[test]
    pub fn parses_mixed_path() -> anyhow::Result<()> {
        let src = "a[b].c[d]";

        let alloc = ASTAllocator::default();
        let mut strings = StringTable::default();
        let result =
            final_parser!((src.as_bytes(), &alloc, &mut strings) => PrefixExpression::parse)?;

        assert_eq!(
            result,
            PrefixExpression::Variable(VarPrefixExpression::TableAccess {
                head: HeadAtom::Name(Ident(0)),
                middle: List::from_slice(&mut [
                    ListNode::new(PrefixAtom::Var(VarAtom::IndexOp(Expression::Variable(
                        &VarPrefixExpression::Name(Ident(1))
                    )))),
                    ListNode::new(PrefixAtom::Var(VarAtom::Name(Ident(2)))),
                ]),
                last: &VarAtom::IndexOp(Expression::Variable(&VarPrefixExpression::Name(Ident(3)))),
            })
        );

        Ok(())
    }

    #[test]
    pub fn parses_parenthetical() -> anyhow::Result<()> {
        let src = "(a)";

        let alloc = ASTAllocator::default();
        let mut strings = StringTable::default();
        let result =
            final_parser!((src.as_bytes(), &alloc, &mut strings) => PrefixExpression::parse)?;

        assert_eq!(
            result,
            PrefixExpression::Parenthesized(&Expression::Variable(&VarPrefixExpression::Name(
                Ident(0)
            )))
        );

        Ok(())
    }

    #[test]
    pub fn parses_parenthetical_mixed_dot() -> anyhow::Result<()> {
        let src = "(a).b";

        let alloc = ASTAllocator::default();
        let mut strings = StringTable::default();
        let result =
            final_parser!((src.as_bytes(), &alloc, &mut strings) => PrefixExpression::parse)?;

        assert_eq!(
            result,
            PrefixExpression::Variable(VarPrefixExpression::TableAccess {
                head: HeadAtom::Parenthesized(&Expression::Variable(&VarPrefixExpression::Name(
                    Ident(0)
                ))),
                middle: List::default(),
                last: &VarAtom::Name(Ident(1)),
            })
        );

        Ok(())
    }

    #[test]
    pub fn parses_parenthetical_mixed_bracket() -> anyhow::Result<()> {
        let src = "(a)[b]";

        let alloc = ASTAllocator::default();
        let mut strings = StringTable::default();
        let result =
            final_parser!((src.as_bytes(), &alloc, &mut strings) => PrefixExpression::parse)?;

        assert_eq!(
            result,
            PrefixExpression::Variable(VarPrefixExpression::TableAccess {
                head: HeadAtom::Parenthesized(&Expression::Variable(&VarPrefixExpression::Name(
                    Ident(0)
                ))),
                middle: List::default(),
                last: &VarAtom::IndexOp(Expression::Variable(&VarPrefixExpression::Name(Ident(1)))),
            })
        );

        Ok(())
    }

    #[test]
    pub fn parses_fn_call() -> anyhow::Result<()> {
        let src = "a()";

        let alloc = ASTAllocator::default();
        let mut strings = StringTable::default();
        let result =
            final_parser!((src.as_bytes(), &alloc, &mut strings) => PrefixExpression::parse)?;

        assert_eq!(
            result,
            PrefixExpression::FnCall(FnCallPrefixExpression::Call {
                head: HeadAtom::Name(Ident(0)),
                args: FunctionAtom::Call(FnArgs::Expressions(Default::default()))
            })
        );

        Ok(())
    }

    #[test]
    pub fn parses_fn_call_tablector() -> anyhow::Result<()> {
        let src = "a{}";

        let alloc = ASTAllocator::default();
        let mut strings = StringTable::default();
        let result =
            final_parser!((src.as_bytes(), &alloc, &mut strings) => PrefixExpression::parse)?;

        assert_eq!(
            result,
            PrefixExpression::FnCall(FnCallPrefixExpression::Call {
                head: HeadAtom::Name(Ident(0)),
                args: FunctionAtom::Call(FnArgs::TableConstructor(TableConstructor {
                    fields: Default::default(),
                }))
            })
        );

        Ok(())
    }

    #[test]
    pub fn parses_fn_call_lit_str() -> anyhow::Result<()> {
        let src = "a\"b\"";

        let alloc = ASTAllocator::default();
        let mut strings = StringTable::default();
        let result =
            final_parser!((src.as_bytes(), &alloc, &mut strings) => PrefixExpression::parse)?;

        assert_eq!(
            result,
            PrefixExpression::FnCall(FnCallPrefixExpression::Call {
                head: HeadAtom::Name(Ident(0)),
                args: FunctionAtom::Call(FnArgs::String(ConstantString(1)))
            })
        );

        Ok(())
    }

    #[test]
    pub fn parses_named_fn_call() -> anyhow::Result<()> {
        let src = "a:foo()";

        let alloc = ASTAllocator::default();
        let mut strings = StringTable::default();
        let result =
            final_parser!((src.as_bytes(), &alloc, &mut strings) => PrefixExpression::parse)?;

        assert_eq!(
            result,
            PrefixExpression::FnCall(FnCallPrefixExpression::Call {
                head: HeadAtom::Name(Ident(0)),
                args: FunctionAtom::MethodCall {
                    name: Ident(1),
                    args: FnArgs::Expressions(Default::default())
                }
            })
        );

        Ok(())
    }

    #[test]
    pub fn parses_named_fn_call_tablector() -> anyhow::Result<()> {
        let src = "a:foo{}";

        let alloc = ASTAllocator::default();
        let mut strings = StringTable::default();
        let result =
            final_parser!((src.as_bytes(), &alloc, &mut strings) => PrefixExpression::parse)?;

        assert_eq!(
            result,
            PrefixExpression::FnCall(FnCallPrefixExpression::Call {
                head: HeadAtom::Name(Ident(0)),
                args: FunctionAtom::MethodCall {
                    name: Ident(1),
                    args: FnArgs::TableConstructor(TableConstructor {
                        fields: Default::default(),
                    })
                }
            })
        );

        Ok(())
    }

    #[test]
    pub fn parses_named_fn_call_lit_str() -> anyhow::Result<()> {
        let src = "a:foo\"b\"";

        let alloc = ASTAllocator::default();
        let mut strings = StringTable::default();
        let result =
            final_parser!((src.as_bytes(), &alloc, &mut strings) => PrefixExpression::parse)?;

        assert_eq!(
            result,
            PrefixExpression::FnCall(FnCallPrefixExpression::Call {
                head: HeadAtom::Name(Ident(0)),
                args: FunctionAtom::MethodCall {
                    name: Ident(1),
                    args: FnArgs::String(ConstantString(2))
                }
            })
        );

        Ok(())
    }
}
