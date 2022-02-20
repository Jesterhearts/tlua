use nom::{
    bytes::complete::tag,
    combinator::{
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
    lua_whitespace0,
    lua_whitespace1,
    ASTAllocator,
    Parse,
    ParseResult,
    Span,
};

#[derive(Debug, PartialEq)]
pub struct WhileLoop<'chunk> {
    pub cond: Expression<'chunk>,
    pub body: Block<'chunk>,
}

impl<'chunk> Parse<'chunk> for WhileLoop<'chunk> {
    fn parse<'src>(input: Span<'src>, alloc: &'chunk ASTAllocator) -> ParseResult<'src, Self> {
        delimited(
            pair(tag("while"), lua_whitespace1),
            map(
                tuple((
                    |input| Expression::parse(input, alloc),
                    value((), delimited(lua_whitespace0, tag("do"), lua_whitespace1)),
                    |input| Block::parse(input, alloc),
                )),
                |(cond, _, body)| Self { cond, body },
            ),
            pair(lua_whitespace0, tag("end")),
        )(input)
    }
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;

    use super::WhileLoop;
    use crate::{
        expressions::Expression,
        ASTAllocator,
        Parse,
        Span,
    };

    #[test]
    pub fn parses_while() -> anyhow::Result<()> {
        let src = "while true do end";

        let alloc = ASTAllocator::default();
        let (remain, result) = WhileLoop::parse(Span::new(src.as_bytes()), &alloc)?;

        assert_eq!(std::str::from_utf8(*remain)?, "");
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