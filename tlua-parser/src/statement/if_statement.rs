use nom::{
    combinator::{
        cut,
        map,
        opt,
    },
    sequence::{
        delimited,
        pair,
        preceded,
        terminated,
        tuple,
    },
};

use crate::{
    block::Block,
    build_list0,
    expressions::Expression,
    identifiers::keyword,
    list::List,
    lua_whitespace0,
    lua_whitespace1,
    ASTAllocator,
    ParseResult,
    Span,
};

#[derive(Debug, PartialEq)]
pub struct If<'chunk> {
    pub cond: Expression<'chunk>,
    pub body: Block<'chunk>,
    pub elif: List<'chunk, ElseIf<'chunk>>,
    pub else_final: Option<Block<'chunk>>,
}

#[derive(Debug, PartialEq)]
pub struct ElseIf<'chunk> {
    pub cond: Expression<'chunk>,
    pub body: Block<'chunk>,
}

impl<'chunk> If<'chunk> {
    pub(crate) fn parser(
        alloc: &'chunk ASTAllocator,
    ) -> impl for<'src> FnMut(Span<'src>) -> ParseResult<'src, If<'chunk>> {
        |input| {
            delimited(
                pair(keyword("if"), lua_whitespace1),
                map(
                    cut(tuple((
                        |input| parse_cond_then_body(input, alloc),
                        build_list0(
                            alloc,
                            map(
                                preceded(
                                    delimited(lua_whitespace0, keyword("elseif"), lua_whitespace1),
                                    |input| parse_cond_then_body(input, alloc),
                                ),
                                |(cond, body)| ElseIf { cond, body },
                            ),
                        ),
                        opt(preceded(
                            delimited(lua_whitespace0, keyword("else"), lua_whitespace1),
                            Block::parser(alloc),
                        )),
                    ))),
                    |((cond, body), elif, else_final)| Self {
                        cond,
                        body,
                        elif,
                        else_final,
                    },
                ),
                pair(lua_whitespace0, keyword("end")),
            )(input)
        }
    }
}

fn parse_cond_then_body<'src, 'chunk>(
    input: Span<'src>,
    alloc: &'chunk ASTAllocator,
) -> ParseResult<'src, (Expression<'chunk>, Block<'chunk>)> {
    pair(
        terminated(
            Expression::parser(alloc),
            delimited(lua_whitespace0, keyword("then"), lua_whitespace0),
        ),
        Block::parser(alloc),
    )(input)
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;

    use super::If;
    use crate::{
        expressions::Expression,
        final_parser,
        list::{
            List,
            ListNode,
        },
        statement::if_statement::ElseIf,
        ASTAllocator,
        Span,
    };

    #[test]
    pub fn parses_if() -> anyhow::Result<()> {
        let src = "if true then end";

        let alloc = ASTAllocator::default();
        let result = final_parser!(Span::new(src.as_bytes()) => If::parser(&alloc))?;

        assert_eq!(
            result,
            If {
                cond: Expression::Bool(true),
                body: Default::default(),
                elif: Default::default(),
                else_final: None,
            }
        );

        Ok(())
    }

    #[test]
    pub fn parses_if_else() -> anyhow::Result<()> {
        let src = "if true then else end";

        let alloc = ASTAllocator::default();
        let result = final_parser!(Span::new(src.as_bytes()) => If::parser(&alloc))?;

        assert_eq!(
            result,
            If {
                cond: Expression::Bool(true),
                body: Default::default(),
                elif: Default::default(),
                else_final: Some(Default::default())
            }
        );

        Ok(())
    }

    #[test]
    pub fn parses_if_elseif_else() -> anyhow::Result<()> {
        let src = "if true then elseif true then else end";

        let alloc = ASTAllocator::default();
        let result = final_parser!(Span::new(src.as_bytes()) => If::parser(&alloc))?;

        assert_eq!(
            result,
            If {
                cond: Expression::Bool(true),
                body: Default::default(),
                elif: List::new(&mut ListNode::new(ElseIf {
                    cond: Expression::Bool(true),
                    body: Default::default(),
                })),
                else_final: Some(Default::default())
            }
        );

        Ok(())
    }
}
