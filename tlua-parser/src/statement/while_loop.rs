use nom::{
    combinator::{
        cut,
        map,
        value,
    },
    sequence::{
        delimited,
        pair,
        tuple,
    },
};

use crate::{
    block::Block,
    expressions::Expression,
    identifiers::keyword,
    lua_whitespace0,
    lua_whitespace1,
    ASTAllocator,
    ParseResult,
    Span,
};

#[derive(Debug, PartialEq)]
pub struct WhileLoop<'chunk> {
    pub cond: Expression<'chunk>,
    pub body: Block<'chunk>,
}

impl<'chunk> WhileLoop<'chunk> {
    pub(crate) fn parser(
        alloc: &'chunk ASTAllocator,
    ) -> impl for<'src> FnMut(Span<'src>) -> ParseResult<'src, WhileLoop<'chunk>> {
        |input| {
            delimited(
                pair(keyword("while"), lua_whitespace0),
                map(
                    cut(tuple((
                        Expression::parser(alloc),
                        value(
                            (),
                            delimited(lua_whitespace0, keyword("do"), lua_whitespace1),
                        ),
                        Block::parser(alloc),
                    ))),
                    |(cond, _, body)| Self { cond, body },
                ),
                pair(lua_whitespace0, keyword("end")),
            )(input)
        }
    }
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;

    use super::WhileLoop;
    use crate::{
        expressions::Expression,
        final_parser,
        ASTAllocator,
        Span,
    };

    #[test]
    pub fn parses_while() -> anyhow::Result<()> {
        let src = "while true do end";

        let alloc = ASTAllocator::default();
        let result = final_parser!(Span::new(src.as_bytes()) => WhileLoop::parser(&alloc))?;

        assert_eq!(
            result,
            WhileLoop {
                cond: Expression::Bool(true),
                body: Default::default()
            }
        );

        Ok(())
    }
}
