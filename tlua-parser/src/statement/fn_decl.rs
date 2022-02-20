use nom::{
    branch::alt,
    bytes::complete::tag,
    combinator::map,
    sequence::{
        pair,
        preceded,
        terminated,
        tuple,
    },
};

use crate::{
    expressions::function_defs::{
        FnBody,
        FnName,
    },
    identifiers::{
        parse_identifier,
        Ident,
    },
    lua_whitespace0,
    lua_whitespace1,
    ASTAllocator,
    Parse,
    ParseResult,
    Span,
};

#[derive(Debug, PartialEq)]
pub enum FnDecl<'chunk> {
    Function {
        name: FnName<'chunk>,
        body: FnBody<'chunk>,
    },
    Local {
        name: Ident,
        body: FnBody<'chunk>,
    },
}

impl<'chunk> Parse<'chunk> for FnDecl<'chunk> {
    fn parse<'src>(input: Span<'src>, alloc: &'chunk ASTAllocator) -> ParseResult<'src, Self> {
        alt((
            preceded(
                tuple((
                    tag("local"),
                    lua_whitespace1,
                    tag("function"),
                    lua_whitespace1,
                )),
                map(
                    pair(
                        terminated(|input| parse_identifier(input, alloc), lua_whitespace0),
                        |input| FnBody::parse(input, alloc),
                    ),
                    |(name, body)| Self::Local { name, body },
                ),
            ),
            preceded(
                pair(tag("function"), lua_whitespace1),
                map(
                    pair(
                        terminated(|input| FnName::parse(input, alloc), lua_whitespace0),
                        |input| FnBody::parse(input, alloc),
                    ),
                    |(name, body)| Self::Function { name, body },
                ),
            ),
        ))(input)
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        expressions::function_defs::{
            FnBody,
            FnName,
            FnParams,
        },
        final_parser,
        list::{
            List,
            ListNode,
        },
        statement::fn_decl::FnDecl,
        ASTAllocator,
        Parse,
        Span,
    };

    #[test]
    pub fn parses_local_fn_def() -> anyhow::Result<()> {
        let src = "local function foo() end";
        let alloc = ASTAllocator::default();

        let result =
            final_parser!(Span::new(src.as_bytes()) => |input| FnDecl::parse(input, &alloc))?;

        assert_eq!(
            result,
            FnDecl::Local {
                name: "foo".into(),
                body: FnBody {
                    params: FnParams {
                        named_params: Default::default(),
                        varargs: false
                    },
                    body: Default::default()
                }
            }
        );

        Ok(())
    }

    #[test]
    pub fn parses_fn_def() -> anyhow::Result<()> {
        let src = "function foo() end";
        let alloc = ASTAllocator::default();

        let result =
            final_parser!(Span::new(src.as_bytes())=> |input| FnDecl::parse(input, &alloc))?;

        assert_eq!(
            result,
            FnDecl::Function {
                name: FnName {
                    path: List::new(&mut ListNode::new("foo".into())),
                    method: None
                },
                body: FnBody {
                    params: FnParams {
                        named_params: Default::default(),
                        varargs: false
                    },
                    body: Default::default()
                }
            }
        );

        Ok(())
    }

    #[test]
    pub fn parses_method_fn_def() -> anyhow::Result<()> {
        let src = "function foo.bar:baz() end";
        let alloc = ASTAllocator::default();

        let result =
            final_parser!(Span::new(src.as_bytes())=> |input| FnDecl::parse(input, &alloc))?;

        assert_eq!(
            result,
            FnDecl::Function {
                name: FnName {
                    path: List::from_slice(&mut [
                        ListNode::new("foo".into()),
                        ListNode::new("bar".into()),
                    ]),
                    method: Some("baz".into())
                },
                body: FnBody {
                    params: FnParams {
                        named_params: Default::default(),
                        varargs: false
                    },
                    body: Default::default()
                }
            }
        );

        Ok(())
    }
}
