use nom::{
    branch::alt,
    bytes::complete::tag,
    combinator::{
        map,
        opt,
        success,
        value,
    },
    sequence::{
        delimited,
        pair,
        preceded,
        terminated,
    },
};

use crate::{
    block::Block,
    identifiers::{
        identifier_list1,
        parse_identifier,
        Ident,
    },
    list::List,
    lua_whitespace0,
    ASTAllocator,
    Parse,
    ParseResult,
    Span,
};

#[derive(Debug, PartialEq)]
pub struct FnParams<'chunk> {
    /// Note that LUA 5.4 doesn't distinguish multiple variables during function
    /// evaluation. So a function like `(a, a) return a + a; end` when
    /// called with `(10, 11)` produces `22` in valid lua.
    pub named_params: List<'chunk, Ident>,
    pub varargs: bool,
}

#[derive(Debug, PartialEq)]
pub struct FnBody<'chunk> {
    pub params: FnParams<'chunk>,
    pub body: Block<'chunk>,
}

#[derive(Debug, PartialEq)]
pub struct FnName<'chunk> {
    pub path: List<'chunk, Ident>,
    pub method: Option<Ident>,
}

impl<'chunk> Parse<'chunk> for FnParams<'chunk> {
    fn parse<'src>(input: Span<'src>, alloc: &'chunk ASTAllocator) -> ParseResult<'src, Self> {
        delimited(
            pair(tag("("), lua_whitespace0),
            map(
                opt(alt((
                    map(
                        pair(
                            |input| identifier_list1(input, alloc),
                            alt((
                                value(
                                    true,
                                    pair(
                                        preceded(lua_whitespace0, tag(",")),
                                        preceded(lua_whitespace0, tag("...")),
                                    ),
                                ),
                                success(false),
                            )),
                        ),
                        |(named_params, varargs)| Self {
                            named_params,
                            varargs,
                        },
                    ),
                    map(value((), tag("...")), |_| Self {
                        named_params: Default::default(),
                        varargs: true,
                    }),
                ))),
                |maybe_params| {
                    maybe_params.unwrap_or_else(|| Self {
                        named_params: Default::default(),
                        varargs: false,
                    })
                },
            ),
            pair(lua_whitespace0, tag(")")),
        )(input)
    }
}

impl<'chunk> Parse<'chunk> for FnBody<'chunk> {
    fn parse<'src>(input: Span<'src>, alloc: &'chunk ASTAllocator) -> ParseResult<'src, Self> {
        map(
            terminated(
                pair(
                    |input| FnParams::parse(input, alloc),
                    preceded(lua_whitespace0, |input| Block::parse(input, alloc)),
                ),
                pair(lua_whitespace0, tag("end")),
            ),
            |(params, body)| Self { params, body },
        )(input)
    }
}

impl<'chunk> Parse<'chunk> for FnName<'chunk> {
    fn parse<'src>(mut input: Span<'src>, alloc: &'chunk ASTAllocator) -> ParseResult<'src, Self> {
        let (remain, head) = parse_identifier(input, alloc)?;
        input = remain;

        let mut path = List::default();
        let current = path.cursor_mut();
        let mut current = current.alloc_insert_advance(alloc, head);

        loop {
            let (remain, maybe_ident) = opt(preceded(
                delimited(lua_whitespace0, tag("."), lua_whitespace0),
                |input| parse_identifier(input, alloc),
            ))(input)?;
            input = remain;

            current = if let Some(next) = maybe_ident {
                current.alloc_insert_advance(alloc, next)
            } else {
                break;
            };
        }

        let (remain, method) = opt(preceded(pair(lua_whitespace0, tag(":")), |input| {
            parse_identifier(input, alloc)
        }))(input)?;

        Ok((remain, Self { path, method }))
    }
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;

    use crate::{
        block::{
            retstat::RetStatement,
            Block,
        },
        expressions::{
            function_defs::{
                FnBody,
                FnParams,
            },
            number::Number,
            Expression,
        },
        list::{
            List,
            ListNode,
        },
        ASTAllocator,
        Parse,
        Span,
    };

    #[test]
    pub fn parses_empty_params() -> anyhow::Result<()> {
        let src = "()";

        let alloc = ASTAllocator::default();
        let (remain, result) = FnParams::parse(Span::new(src.as_bytes()), &alloc)?;

        assert_eq!(std::str::from_utf8(*remain)?, "");
        assert_eq!(
            result,
            FnParams {
                named_params: Default::default(),
                varargs: false,
            }
        );

        Ok(())
    }

    #[test]
    pub fn parses_params_no_varargs() -> anyhow::Result<()> {
        let src = "(a)";

        let alloc = ASTAllocator::default();
        let (remain, result) = FnParams::parse(Span::new(src.as_bytes()), &alloc)?;

        assert_eq!(std::str::from_utf8(*remain)?, "");
        assert_eq!(
            result,
            FnParams {
                named_params: List::new(&mut ListNode::new("a".into())),
                varargs: false,
            }
        );

        Ok(())
    }

    #[test]
    pub fn parses_params_only_varargs() -> anyhow::Result<()> {
        let src = "(...)";

        let alloc = ASTAllocator::default();
        let (remain, result) = FnParams::parse(Span::new(src.as_bytes()), &alloc)?;

        assert_eq!(std::str::from_utf8(*remain)?, "");
        assert_eq!(
            result,
            FnParams {
                named_params: Default::default(),
                varargs: true,
            }
        );

        Ok(())
    }

    #[test]
    pub fn parses_params_trailing_varags() -> anyhow::Result<()> {
        let src = "(a,b, c, ...)";

        let alloc = ASTAllocator::default();
        let (remain, result) = FnParams::parse(Span::new(src.as_bytes()), &alloc)?;

        assert_eq!(std::str::from_utf8(*remain)?, "");
        assert_eq!(
            result,
            FnParams {
                named_params: List::from_slice(&mut [
                    ListNode::new("a".into()),
                    ListNode::new("b".into()),
                    ListNode::new("c".into())
                ]),
                varargs: true,
            }
        );

        Ok(())
    }

    #[test]
    pub fn parses_func_body() -> anyhow::Result<()> {
        let src = "() return 10 end";

        let alloc = ASTAllocator::default();
        let (remain, result) = FnBody::parse(Span::new(src.as_bytes()), &alloc)?;

        assert_eq!(std::str::from_utf8(*remain)?, "");
        assert_eq!(
            result,
            FnBody {
                params: FnParams {
                    named_params: Default::default(),
                    varargs: false,
                },
                body: Block {
                    statements: Default::default(),
                    ret: Some(RetStatement {
                        expressions: List::new(&mut ListNode::new(Expression::Number(
                            Number::Integer(10)
                        )))
                    })
                }
            }
        );
        Ok(())
    }
}
