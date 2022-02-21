use nom::{
    combinator::{
        cut,
        map,
        value,
    },
    sequence::{
        delimited,
        pair,
        preceded,
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
pub struct RepeatLoop<'chunk> {
    pub body: Block<'chunk>,
    pub terminator: Expression<'chunk>,
}

impl<'chunk> RepeatLoop<'chunk> {
    pub(crate) fn parser(
        alloc: &'chunk ASTAllocator,
    ) -> impl for<'src> FnMut(Span<'src>) -> ParseResult<'src, RepeatLoop<'chunk>> {
        |input| {
            preceded(
                pair(keyword("repeat"), lua_whitespace1),
                map(
                    cut(tuple((
                        Block::parser(alloc),
                        value(
                            (),
                            delimited(lua_whitespace0, keyword("until"), lua_whitespace0),
                        ),
                        Expression::parser(alloc),
                    ))),
                    |(body, _, terminator)| Self { body, terminator },
                ),
            )(input)
        }
    }
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;

    use super::RepeatLoop;
    use crate::{
        expressions::Expression,
        final_parser,
        ASTAllocator,
        Span,
    };

    #[test]
    pub fn parses_repeat() -> anyhow::Result<()> {
        let src = "repeat until true";

        let alloc = ASTAllocator::default();
        let result = final_parser!(Span::new(src.as_bytes())=> RepeatLoop::parser( &alloc))?;

        assert_eq!(
            result,
            RepeatLoop {
                body: Default::default(),
                terminator: Expression::Bool(true)
            }
        );

        Ok(())
    }
}
