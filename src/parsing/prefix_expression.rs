use nom::{
    branch::alt,
    bytes::complete::tag,
    combinator::{
        map,
        opt,
    },
    sequence::{
        delimited,
        pair,
        preceded,
    },
};
use tracing::instrument;

use crate::{
    ast::{
        expressions::Expression,
        prefix_expression::{
            function_calls::FnArgs,
            FnCallPrefixExpression,
            FunctionAtom,
            HeadAtom,
            PrefixAtom,
            PrefixExpression,
            VarAtom,
            VarPrefixExpression,
        },
    },
    list::List,
    parsing::{
        identifiers::parse_identifier,
        lua_whitespace0,
        ASTAllocator,
        Parse,
        ParseResult,
        Span,
    },
};

pub mod function_calls;

impl<'chunk> Parse<'chunk> for PrefixExpression<'chunk> {
    #[instrument(level = "trace", name = "prefix_expr", skip(input, alloc))]
    fn parse<'src>(input: Span<'src>, alloc: &'chunk ASTAllocator) -> ParseResult<'src, Self> {
        // Prefix expressions must start with either a Name or a parenthesized
        // expresssion - all other forms of prefix expressions require a preceding
        // prefix expression.
        let (mut input, head) = alt((
            map(|input| parse_identifier(input, alloc), HeadAtom::Name),
            map(
                delimited(
                    pair(tag("("), lua_whitespace0),
                    |input| Expression::parse(input, alloc),
                    pair(lua_whitespace0, tag(")")),
                ),
                |expr| HeadAtom::Parenthesized(alloc.alloc(expr)),
            ),
        ))(input)?;

        // See if there is another expression after our head atom.
        let (remain, next) = parse_impl(input, alloc)?;
        input = remain;

        let current = if let Some(next) = next {
            next
        } else {
            return Ok((
                input,
                match head {
                    HeadAtom::Name(ident) => Self::Variable(VarPrefixExpression::Name(ident)),
                    HeadAtom::Parenthesized(expr) => Self::Parenthesized(expr),
                },
            ));
        };

        // See if this is a greater than length 2 prefix expression.
        let (remain, next) = parse_impl(input, alloc)?;
        input = remain;

        let mut middle = List::default();

        let (mut previous, mut current) = if let Some(next) = next {
            // We have at least 3 prefix expressions so we will fill out both the head,
            // body, and tail of the expression.
            (
                middle.cursor_mut().alloc_insert_advance(alloc, current),
                next,
            )
        } else {
            // This is a length 2 prefix expression and we want to
            // populate just the head and tail portions of the list. We divide out
            // these cases so we don't have to handle e.g. a function expression
            // with a possible var expression terminating it when processing the AST
            // as that would be obviously impossible.
            return Ok((
                input,
                match current {
                    PrefixAtom::Var(v) => Self::Variable(VarPrefixExpression::TableAccess {
                        head,
                        middle,
                        last: alloc.alloc(v),
                    }),
                    PrefixAtom::Function(f) => {
                        Self::FnCall(FnCallPrefixExpression::Call { head, args: f })
                    }
                },
            ));
        };

        loop {
            let (remain, maybe_next) = parse_impl(input, alloc)?;
            input = remain;

            current = if let Some(next) = maybe_next {
                previous = previous.alloc_insert_advance(alloc, current);
                next
            } else {
                return Ok((
                    input,
                    match current {
                        PrefixAtom::Var(var) => Self::Variable(VarPrefixExpression::TableAccess {
                            head,
                            middle,
                            last: alloc.alloc(var),
                        }),
                        PrefixAtom::Function(f) => Self::FnCall(FnCallPrefixExpression::CallPath {
                            head,
                            middle,
                            last: f,
                        }),
                    },
                ));
            }
        }
    }
}

#[instrument(level = "trace", name = "atom", skip(input, alloc))]
fn parse_impl<'src, 'chunk>(
    input: Span<'src>,
    alloc: &'chunk ASTAllocator,
) -> ParseResult<'src, Option<PrefixAtom<'chunk>>> {
    opt(preceded(
        lua_whitespace0,
        alt((
            |input| parse_index_op(input, alloc),
            |input| parse_dot_name(input, alloc),
            |input| parse_method_call(input, alloc),
            |input| parse_call(input, alloc),
        )),
    ))(input)
}

#[instrument(level = "trace", name = "index_op", skip(input, alloc))]
fn parse_index_op<'src, 'chunk>(
    input: Span<'src>,
    alloc: &'chunk ASTAllocator,
) -> ParseResult<'src, PrefixAtom<'chunk>> {
    delimited(
        pair(tag("["), lua_whitespace0),
        map(
            |input| Expression::parse(input, alloc),
            |expr| PrefixAtom::Var(VarAtom::IndexOp(expr)),
        ),
        pair(lua_whitespace0, tag("]")),
    )(input)
}

#[instrument(level = "trace", name = "dot_name", skip(input, alloc))]
fn parse_dot_name<'src, 'chunk>(
    input: Span<'src>,
    alloc: &'chunk ASTAllocator,
) -> ParseResult<'src, PrefixAtom<'chunk>> {
    preceded(
        pair(tag("."), lua_whitespace0),
        map(
            |input| parse_identifier(input, alloc),
            |ident| PrefixAtom::Var(VarAtom::Name(ident)),
        ),
    )(input)
}

#[instrument(level = "trace", name = "call", skip(input, alloc))]
fn parse_call<'src, 'chunk>(
    input: Span<'src>,
    alloc: &'chunk ASTAllocator,
) -> ParseResult<'src, PrefixAtom<'chunk>> {
    map(
        |input| FnArgs::parse(input, alloc),
        |args| PrefixAtom::Function(FunctionAtom::Call(args)),
    )(input)
}

#[instrument(level = "trace", name = "method_call", skip(input, alloc))]
fn parse_method_call<'src, 'chunk>(
    input: Span<'src>,
    alloc: &'chunk ASTAllocator,
) -> ParseResult<'src, PrefixAtom<'chunk>> {
    preceded(
        tag(":"),
        map(
            pair(
                |input| parse_identifier(input, alloc),
                |input| FnArgs::parse(input, alloc),
            ),
            |(ident, args)| PrefixAtom::Function(FunctionAtom::MethodCall { name: ident, args }),
        ),
    )(input)
}

#[cfg(test)]
mod tests {
    use nom_supreme::final_parser::final_parser;
    use pretty_assertions::assert_eq;

    use super::PrefixExpression;
    use crate::{
        ast::{
            expressions::{
                tables::TableConstructor,
                Expression,
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
        },
        list::{
            List,
            ListNode,
        },
        parsing::{
            ASTAllocator,
            InternalLuaParseError,
            LuaParseError,
            Parse,
            Span,
        },
    };

    #[test]
    pub(crate) fn parses_dotted_path() -> anyhow::Result<()> {
        let src = "a.b.c";

        let alloc = ASTAllocator::default();
        let result =
            final_parser(|input| PrefixExpression::parse(input, &alloc))(Span::new(src.as_bytes()))
                .map_err(|e: InternalLuaParseError| e.map_locations(LuaParseError::from))?;

        assert_eq!(
            result,
            PrefixExpression::Variable(VarPrefixExpression::TableAccess {
                head: HeadAtom::Name("a".into()),
                middle: List::new(&mut ListNode::new(PrefixAtom::Var(VarAtom::Name(
                    "b".into()
                )))),
                last: &VarAtom::Name("c".into())
            })
        );

        Ok(())
    }

    #[test]
    pub(crate) fn parses_bracket_path() -> anyhow::Result<()> {
        let src = "a[b][c]";

        let alloc = ASTAllocator::default();
        let result =
            final_parser(|input| PrefixExpression::parse(input, &alloc))(Span::new(src.as_bytes()))
                .map_err(|e: InternalLuaParseError| e.map_locations(LuaParseError::from))?;

        assert_eq!(
            result,
            PrefixExpression::Variable(VarPrefixExpression::TableAccess {
                head: HeadAtom::Name("a".into()),
                middle: List::new(&mut ListNode::new(PrefixAtom::Var(VarAtom::IndexOp(
                    Expression::Variable(&VarPrefixExpression::Name("b".into()))
                ))),),
                last: &VarAtom::IndexOp(Expression::Variable(&VarPrefixExpression::Name(
                    "c".into()
                ))),
            })
        );

        Ok(())
    }

    #[test]
    pub(crate) fn parses_mixed_path() -> anyhow::Result<()> {
        let src = "a[b].c[d]";

        let alloc = ASTAllocator::default();
        let result =
            final_parser(|input| PrefixExpression::parse(input, &alloc))(Span::new(src.as_bytes()))
                .map_err(|e: InternalLuaParseError| e.map_locations(LuaParseError::from))?;

        assert_eq!(
            result,
            PrefixExpression::Variable(VarPrefixExpression::TableAccess {
                head: HeadAtom::Name("a".into()),
                middle: List::from_slice(&mut [
                    ListNode::new(PrefixAtom::Var(VarAtom::IndexOp(Expression::Variable(
                        &VarPrefixExpression::Name("b".into())
                    )))),
                    ListNode::new(PrefixAtom::Var(VarAtom::Name("c".into()))),
                ]),
                last: &VarAtom::IndexOp(Expression::Variable(&VarPrefixExpression::Name(
                    "d".into()
                ))),
            })
        );

        Ok(())
    }

    #[test]
    pub(crate) fn parses_parenthetical() -> anyhow::Result<()> {
        let src = "(a)";

        let alloc = ASTAllocator::default();
        let result =
            final_parser(|input| PrefixExpression::parse(input, &alloc))(Span::new(src.as_bytes()))
                .map_err(|e: InternalLuaParseError| e.map_locations(LuaParseError::from))?;

        assert_eq!(
            result,
            PrefixExpression::Parenthesized(&Expression::Variable(&VarPrefixExpression::Name(
                "a".into()
            )))
        );

        Ok(())
    }

    #[test]
    pub(crate) fn parses_parenthetical_mixed_dot() -> anyhow::Result<()> {
        let src = "(a).b";

        let alloc = ASTAllocator::default();
        let result =
            final_parser(|input| PrefixExpression::parse(input, &alloc))(Span::new(src.as_bytes()))
                .map_err(|e: InternalLuaParseError| e.map_locations(LuaParseError::from))?;

        assert_eq!(
            result,
            PrefixExpression::Variable(VarPrefixExpression::TableAccess {
                head: HeadAtom::Parenthesized(&Expression::Variable(&VarPrefixExpression::Name(
                    "a".into()
                ))),
                middle: List::default(),
                last: &VarAtom::Name("b".into()),
            })
        );

        Ok(())
    }

    #[test]
    pub(crate) fn parses_parenthetical_mixed_bracket() -> anyhow::Result<()> {
        let src = "(a)[b]";

        let alloc = ASTAllocator::default();
        let result =
            final_parser(|input| PrefixExpression::parse(input, &alloc))(Span::new(src.as_bytes()))
                .map_err(|e: InternalLuaParseError| e.map_locations(LuaParseError::from))?;

        assert_eq!(
            result,
            PrefixExpression::Variable(VarPrefixExpression::TableAccess {
                head: HeadAtom::Parenthesized(&Expression::Variable(&VarPrefixExpression::Name(
                    "a".into()
                ))),
                middle: List::default(),
                last: &VarAtom::IndexOp(Expression::Variable(&VarPrefixExpression::Name(
                    "b".into()
                ))),
            })
        );

        Ok(())
    }

    #[test]
    pub(crate) fn parses_fn_call() -> anyhow::Result<()> {
        let src = "a()";

        let alloc = ASTAllocator::default();
        let result =
            final_parser(|input| PrefixExpression::parse(input, &alloc))(Span::new(src.as_bytes()))
                .map_err(|e: InternalLuaParseError| e.map_locations(LuaParseError::from))?;

        assert_eq!(
            result,
            PrefixExpression::FnCall(FnCallPrefixExpression::Call {
                head: HeadAtom::Name("a".into()),
                args: FunctionAtom::Call(FnArgs::Expressions(Default::default()))
            })
        );

        Ok(())
    }

    #[test]
    pub(crate) fn parses_fn_call_tablector() -> anyhow::Result<()> {
        let src = "a{}";

        let alloc = ASTAllocator::default();
        let result =
            final_parser(|input| PrefixExpression::parse(input, &alloc))(Span::new(src.as_bytes()))
                .map_err(|e: InternalLuaParseError| e.map_locations(LuaParseError::from))?;

        assert_eq!(
            result,
            PrefixExpression::FnCall(FnCallPrefixExpression::Call {
                head: HeadAtom::Name("a".into()),
                args: FunctionAtom::Call(FnArgs::TableConstructor(TableConstructor {
                    fields: Default::default()
                }))
            })
        );

        Ok(())
    }

    #[test]
    pub(crate) fn parses_fn_call_lit_str() -> anyhow::Result<()> {
        let src = "a\"b\"";

        let alloc = ASTAllocator::default();
        let result =
            final_parser(|input| PrefixExpression::parse(input, &alloc))(Span::new(src.as_bytes()))
                .map_err(|e: InternalLuaParseError| e.map_locations(LuaParseError::from))?;

        assert_eq!(
            result,
            PrefixExpression::FnCall(FnCallPrefixExpression::Call {
                head: HeadAtom::Name("a".into()),
                args: FunctionAtom::Call(FnArgs::String("b".into()))
            })
        );

        Ok(())
    }

    #[test]
    pub(crate) fn parses_named_fn_call() -> anyhow::Result<()> {
        let src = "a:foo()";

        let alloc = ASTAllocator::default();
        let result =
            final_parser(|input| PrefixExpression::parse(input, &alloc))(Span::new(src.as_bytes()))
                .map_err(|e: InternalLuaParseError| e.map_locations(LuaParseError::from))?;

        assert_eq!(
            result,
            PrefixExpression::FnCall(FnCallPrefixExpression::Call {
                head: HeadAtom::Name("a".into()),
                args: FunctionAtom::MethodCall {
                    name: "foo".into(),
                    args: FnArgs::Expressions(Default::default())
                }
            })
        );

        Ok(())
    }

    #[test]
    pub(crate) fn parses_named_fn_call_tablector() -> anyhow::Result<()> {
        let src = "a:foo{}";

        let alloc = ASTAllocator::default();
        let result =
            final_parser(|input| PrefixExpression::parse(input, &alloc))(Span::new(src.as_bytes()))
                .map_err(|e: InternalLuaParseError| e.map_locations(LuaParseError::from))?;

        assert_eq!(
            result,
            PrefixExpression::FnCall(FnCallPrefixExpression::Call {
                head: HeadAtom::Name("a".into()),
                args: FunctionAtom::MethodCall {
                    name: "foo".into(),
                    args: FnArgs::TableConstructor(TableConstructor {
                        fields: Default::default()
                    })
                }
            })
        );

        Ok(())
    }

    #[test]
    pub(crate) fn parses_named_fn_call_lit_str() -> anyhow::Result<()> {
        let src = "a:foo\"b\"";

        let alloc = ASTAllocator::default();
        let result =
            final_parser(|input| PrefixExpression::parse(input, &alloc))(Span::new(src.as_bytes()))
                .map_err(|e: InternalLuaParseError| e.map_locations(LuaParseError::from))?;

        assert_eq!(
            result,
            PrefixExpression::FnCall(FnCallPrefixExpression::Call {
                head: HeadAtom::Name("a".into()),
                args: FunctionAtom::MethodCall {
                    name: "foo".into(),
                    args: FnArgs::String("b".into())
                }
            })
        );

        Ok(())
    }
}
