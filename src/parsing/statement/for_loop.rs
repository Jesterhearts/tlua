use nom::{
    bytes::complete::tag,
    combinator::{
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
use tracing::instrument;

use crate::{
    ast::{
        block::Block,
        expressions::Expression,
        statement::for_loop::ForLoop,
    },
    parsing::{
        identifiers::parse_identifier,
        lua_whitespace0,
        lua_whitespace1,
        ASTAllocator,
        Parse,
        ParseResult,
        Span,
    },
};

impl<'chunk> Parse<'chunk> for ForLoop<'chunk> {
    #[instrument(level = "trace", name = "for", skip(input, alloc))]
    fn parse<'src>(input: Span<'src>, alloc: &'chunk ASTAllocator) -> ParseResult<'src, Self> {
        preceded(
            pair(tag("for"), lua_whitespace1),
            map(
                tuple((
                    terminated(
                        |input| parse_identifier(input, alloc),
                        delimited(lua_whitespace0, tag("="), lua_whitespace0),
                    ),
                    terminated(
                        |input| Expression::parse(input, alloc),
                        delimited(lua_whitespace0, tag(","), lua_whitespace0),
                    ),
                    |input| Expression::parse(input, alloc),
                    opt(preceded(
                        delimited(lua_whitespace0, tag(","), lua_whitespace0),
                        |input| Expression::parse(input, alloc),
                    )),
                    delimited(
                        delimited(lua_whitespace0, tag("do"), lua_whitespace1),
                        |input| Block::parse(input, alloc),
                        preceded(lua_whitespace0, tag("end")),
                    ),
                )),
                |(var, init, condition, increment, body)| Self {
                    var,
                    init,
                    condition,
                    increment,
                    body,
                },
            ),
        )(input)
    }
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;

    use super::ForLoop;
    use crate::{
        ast::expressions::Expression,
        parsing::{
            ASTAllocator,
            Parse,
            Span,
        },
        vm::Number,
    };

    #[test]
    pub(crate) fn parses_for() -> anyhow::Result<()> {
        let src = "for a = 0, 10 do end";

        let alloc = ASTAllocator::default();
        let (remain, result) = ForLoop::parse(Span::new(src.as_bytes()), &alloc)?;

        assert_eq!(std::str::from_utf8(*remain)?, "");
        assert_eq!(
            result,
            ForLoop {
                var: "a".into(),
                init: Expression::Number(Number::Integer(0)),
                condition: Expression::Number(Number::Integer(10)),
                increment: None,
                body: Default::default()
            }
        );

        Ok(())
    }
}
