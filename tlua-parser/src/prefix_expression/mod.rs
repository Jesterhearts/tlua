use crate::{
    combinators::parse_list0_split_tail,
    expressions::Expression,
    identifiers::Ident,
    lexer::Token,
    list::List,
    token_subset,
    ASTAllocator,
    ParseError,
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

token_subset! {
    PrefixHeadToken {
        Token::Ident,
        Token::LParen,
        Error(SyntaxError::ExpectedToken2(Token::Ident, Token::LParen))
    }
}

impl<'chunk> HeadAtom<'chunk> {
    fn try_parse(
        lexer: &mut PeekableLexer,
        alloc: &'chunk ASTAllocator,
    ) -> Result<Option<Self>, ParseError> {
        let token = if let Some(token) = PrefixHeadToken::next(lexer) {
            token
        } else {
            return Ok(None);
        };

        match token.as_ref() {
            PrefixHeadToken::Ident => Ok(Some(Self::Name(lexer.strings.add_ident(token.src)))),
            PrefixHeadToken::LParen => Expression::parse(lexer, alloc).and_then(|expr| {
                lexer.expecting_token(Token::RParen)?;
                Ok(Some(Self::Parenthesized(alloc.alloc(expr))))
            }),
        }
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

token_subset! {
    PrefixToken {
        Token::LBracket,
        Token::Period,
        Token::Colon,
        Error(SyntaxError::ExpectedVarOrCall)
    }
}

impl<'chunk> PrefixAtom<'chunk> {
    pub(crate) fn try_parse(
        lexer: &mut PeekableLexer,
        alloc: &'chunk ASTAllocator,
    ) -> Result<Option<Self>, ParseError> {
        let token = if let Some(token) = PrefixToken::next(lexer) {
            token
        } else if let Some(args) = FnArgs::try_parse(lexer, alloc)? {
            return Ok(Some(Self::Function(FunctionAtom::Call(args))));
        } else {
            return Ok(None);
        };

        match token.as_ref() {
            PrefixToken::LBracket => Expression::parse(lexer, alloc).and_then(|expr| {
                lexer
                    .expecting_token(Token::RBracket)
                    .map(|_| Some(Self::Var(VarAtom::IndexOp(expr))))
            }),
            PrefixToken::Period => {
                Ident::parse(lexer, alloc).map(|ident| Some(Self::Var(VarAtom::Name(ident))))
            }
            PrefixToken::Colon => Ident::parse(lexer, alloc).and_then(|name| {
                FnArgs::parse(lexer, alloc)
                    .map(|args| Some(Self::Function(FunctionAtom::MethodCall { name, args })))
            }),
        }
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

impl<'chunk>
    From<(
        HeadAtom<'chunk>,
        List<'chunk, PrefixAtom<'chunk>>,
        FunctionAtom<'chunk>,
    )> for FnCallPrefixExpression<'chunk>
{
    fn from(
        (head, middle, last): (
            HeadAtom<'chunk>,
            List<'chunk, PrefixAtom<'chunk>>,
            FunctionAtom<'chunk>,
        ),
    ) -> Self {
        if middle.is_empty() {
            Self::Call { head, args: last }
        } else {
            Self::CallPath { head, middle, last }
        }
    }
}

#[derive(Debug, PartialEq)]
pub(crate) enum PrefixExpression<'chunk> {
    Variable(VarPrefixExpression<'chunk>),
    FnCall(FnCallPrefixExpression<'chunk>),
    Parenthesized(&'chunk Expression<'chunk>),
}

impl<'chunk> PrefixExpression<'chunk> {
    pub(crate) fn try_parse(
        lexer: &mut PeekableLexer,
        alloc: &'chunk ASTAllocator,
    ) -> Result<Option<Self>, ParseError> {
        if let Some(head) = HeadAtom::try_parse(lexer, alloc)? {
            Self::parse_remaining(head, lexer, alloc).map(Some)
        } else {
            Ok(None)
        }
    }

    pub(crate) fn parse_remaining(
        head: HeadAtom<'chunk>,
        lexer: &mut PeekableLexer,
        alloc: &'chunk ASTAllocator,
    ) -> Result<Self, ParseError> {
        let rest = parse_list0_split_tail(lexer, alloc, PrefixAtom::try_parse)?;

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
            match head {
                HeadAtom::Name(name) => Ok(Self::Variable(VarPrefixExpression::Name(name))),
                HeadAtom::Parenthesized(expr) => Ok(Self::Parenthesized(expr)),
            }
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
            final_parser!((src.as_bytes(), &alloc, &mut strings) => PrefixExpression::try_parse)?;

        assert_eq!(
            result,
            Some(PrefixExpression::Variable(
                VarPrefixExpression::TableAccess {
                    head: HeadAtom::Name(Ident(0)),
                    middle: List::new(&mut ListNode::new(PrefixAtom::Var(VarAtom::Name(Ident(1))))),
                    last: &VarAtom::Name(Ident(2))
                }
            ))
        );

        Ok(())
    }

    #[test]
    pub fn parses_bracket_path() -> anyhow::Result<()> {
        let src = "a[b][c]";

        let alloc = ASTAllocator::default();
        let mut strings = StringTable::default();
        let result =
            final_parser!((src.as_bytes(), &alloc, &mut strings) => PrefixExpression::try_parse)?;

        assert_eq!(
            result,
            Some(PrefixExpression::Variable(
                VarPrefixExpression::TableAccess {
                    head: HeadAtom::Name(Ident(0)),
                    middle: List::new(&mut ListNode::new(PrefixAtom::Var(VarAtom::IndexOp(
                        Expression::Variable(&VarPrefixExpression::Name(Ident(1)))
                    ))),),
                    last: &VarAtom::IndexOp(Expression::Variable(&VarPrefixExpression::Name(
                        Ident(2)
                    ))),
                }
            ))
        );

        Ok(())
    }

    #[test]
    pub fn parses_mixed_path() -> anyhow::Result<()> {
        let src = "a[b].c[d]";

        let alloc = ASTAllocator::default();
        let mut strings = StringTable::default();
        let result =
            final_parser!((src.as_bytes(), &alloc, &mut strings) => PrefixExpression::try_parse)?;

        assert_eq!(
            result,
            Some(PrefixExpression::Variable(
                VarPrefixExpression::TableAccess {
                    head: HeadAtom::Name(Ident(0)),
                    middle: List::from_slice(&mut [
                        ListNode::new(PrefixAtom::Var(VarAtom::IndexOp(Expression::Variable(
                            &VarPrefixExpression::Name(Ident(1))
                        )))),
                        ListNode::new(PrefixAtom::Var(VarAtom::Name(Ident(2)))),
                    ]),
                    last: &VarAtom::IndexOp(Expression::Variable(&VarPrefixExpression::Name(
                        Ident(3)
                    ))),
                }
            ))
        );

        Ok(())
    }

    #[test]
    pub fn parses_parenthetical() -> anyhow::Result<()> {
        let src = "(a)";

        let alloc = ASTAllocator::default();
        let mut strings = StringTable::default();
        let result =
            final_parser!((src.as_bytes(), &alloc, &mut strings) => PrefixExpression::try_parse)?;

        assert_eq!(
            result,
            Some(PrefixExpression::Parenthesized(&Expression::Variable(
                &VarPrefixExpression::Name(Ident(0))
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
            final_parser!((src.as_bytes(), &alloc, &mut strings) => PrefixExpression::try_parse)?;

        assert_eq!(
            result,
            Some(PrefixExpression::Variable(
                VarPrefixExpression::TableAccess {
                    head: HeadAtom::Parenthesized(&Expression::Variable(
                        &VarPrefixExpression::Name(Ident(0))
                    )),
                    middle: List::default(),
                    last: &VarAtom::Name(Ident(1)),
                }
            ))
        );

        Ok(())
    }

    #[test]
    pub fn parses_parenthetical_mixed_bracket() -> anyhow::Result<()> {
        let src = "(a)[b]";

        let alloc = ASTAllocator::default();
        let mut strings = StringTable::default();
        let result =
            final_parser!((src.as_bytes(), &alloc, &mut strings) => PrefixExpression::try_parse)?;

        assert_eq!(
            result,
            Some(PrefixExpression::Variable(
                VarPrefixExpression::TableAccess {
                    head: HeadAtom::Parenthesized(&Expression::Variable(
                        &VarPrefixExpression::Name(Ident(0))
                    )),
                    middle: List::default(),
                    last: &VarAtom::IndexOp(Expression::Variable(&VarPrefixExpression::Name(
                        Ident(1)
                    ))),
                }
            ))
        );

        Ok(())
    }

    #[test]
    pub fn parses_fn_call() -> anyhow::Result<()> {
        let src = "a()";

        let alloc = ASTAllocator::default();
        let mut strings = StringTable::default();
        let result =
            final_parser!((src.as_bytes(), &alloc, &mut strings) => PrefixExpression::try_parse)?;

        assert_eq!(
            result,
            Some(PrefixExpression::FnCall(FnCallPrefixExpression::Call {
                head: HeadAtom::Name(Ident(0)),
                args: FunctionAtom::Call(FnArgs::Expressions(Default::default()))
            }))
        );

        Ok(())
    }

    #[test]
    pub fn parses_fn_call_tablector() -> anyhow::Result<()> {
        let src = "a{}";

        let alloc = ASTAllocator::default();
        let mut strings = StringTable::default();
        let result =
            final_parser!((src.as_bytes(), &alloc, &mut strings) => PrefixExpression::try_parse)?;

        assert_eq!(
            result,
            Some(PrefixExpression::FnCall(FnCallPrefixExpression::Call {
                head: HeadAtom::Name(Ident(0)),
                args: FunctionAtom::Call(FnArgs::TableConstructor(TableConstructor {
                    fields: Default::default(),
                }))
            }))
        );

        Ok(())
    }

    #[test]
    pub fn parses_fn_call_lit_str() -> anyhow::Result<()> {
        let src = "a\"b\"";

        let alloc = ASTAllocator::default();
        let mut strings = StringTable::default();
        let result =
            final_parser!((src.as_bytes(), &alloc, &mut strings) => PrefixExpression::try_parse)?;

        assert_eq!(
            result,
            Some(PrefixExpression::FnCall(FnCallPrefixExpression::Call {
                head: HeadAtom::Name(Ident(0)),
                args: FunctionAtom::Call(FnArgs::String(ConstantString(1)))
            }))
        );

        Ok(())
    }

    #[test]
    pub fn parses_named_fn_call() -> anyhow::Result<()> {
        let src = "a:foo()";

        let alloc = ASTAllocator::default();
        let mut strings = StringTable::default();
        let result =
            final_parser!((src.as_bytes(), &alloc, &mut strings) => PrefixExpression::try_parse)?;

        assert_eq!(
            result,
            Some(PrefixExpression::FnCall(FnCallPrefixExpression::Call {
                head: HeadAtom::Name(Ident(0)),
                args: FunctionAtom::MethodCall {
                    name: Ident(1),
                    args: FnArgs::Expressions(Default::default())
                }
            }))
        );

        Ok(())
    }

    #[test]
    pub fn parses_named_fn_call_tablector() -> anyhow::Result<()> {
        let src = "a:foo{}";

        let alloc = ASTAllocator::default();
        let mut strings = StringTable::default();
        let result =
            final_parser!((src.as_bytes(), &alloc, &mut strings) => PrefixExpression::try_parse)?;

        assert_eq!(
            result,
            Some(PrefixExpression::FnCall(FnCallPrefixExpression::Call {
                head: HeadAtom::Name(Ident(0)),
                args: FunctionAtom::MethodCall {
                    name: Ident(1),
                    args: FnArgs::TableConstructor(TableConstructor {
                        fields: Default::default(),
                    })
                }
            }))
        );

        Ok(())
    }

    #[test]
    pub fn parses_named_fn_call_lit_str() -> anyhow::Result<()> {
        let src = "a:foo\"b\"";

        let alloc = ASTAllocator::default();
        let mut strings = StringTable::default();
        let result =
            final_parser!((src.as_bytes(), &alloc, &mut strings) => PrefixExpression::try_parse)?;

        assert_eq!(
            result,
            Some(PrefixExpression::FnCall(FnCallPrefixExpression::Call {
                head: HeadAtom::Name(Ident(0)),
                args: FunctionAtom::MethodCall {
                    name: Ident(1),
                    args: FnArgs::String(ConstantString(2))
                }
            }))
        );

        Ok(())
    }
}
