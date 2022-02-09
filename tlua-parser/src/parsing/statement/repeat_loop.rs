use nom::{
    bytes::complete::tag,
    combinator::{
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
    ast::{
        block::Block,
        expressions::Expression,
        statement::repeat_loop::RepeatLoop,
    },
    parsing::{
        lua_whitespace0,
        lua_whitespace1,
        ASTAllocator,
        Parse,
        ParseResult,
        Span,
    },
};

impl<'chunk> Parse<'chunk> for RepeatLoop<'chunk> {
    fn parse<'src>(input: Span<'src>, alloc: &'chunk ASTAllocator) -> ParseResult<'src, Self> {
        preceded(
            pair(tag("repeat"), lua_whitespace1),
            map(
                tuple((
                    |input| Block::parse(input, alloc),
                    value(
                        (),
                        delimited(lua_whitespace0, tag("until"), lua_whitespace1),
                    ),
                    |input| Expression::parse(input, alloc),
                )),
                |(body, _, terminator)| Self { body, terminator },
            ),
        )(input)
    }
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;

    use super::RepeatLoop;
    use crate::{
        ast::expressions::Expression,
        parsing::{
            ASTAllocator,
            Parse,
            Span,
        },
    };

    #[test]
    pub fn parses_repeat() -> anyhow::Result<()> {
        let src = "repeat until true";

        let alloc = ASTAllocator::default();
        let (remain, result) = RepeatLoop::parse(Span::new(src.as_bytes()), &alloc)?;

        assert_eq!(std::str::from_utf8(*remain)?, "");
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
