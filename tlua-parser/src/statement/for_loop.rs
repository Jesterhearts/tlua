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

use crate::{
    block::Block,
    expressions::Expression,
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
pub struct ForLoop<'chunk> {
    pub var: Ident,
    pub init: Expression<'chunk>,
    pub condition: Expression<'chunk>,
    pub increment: Option<Expression<'chunk>>,
    pub body: Block<'chunk>,
}

impl<'chunk> Parse<'chunk> for ForLoop<'chunk> {
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
        expressions::{
            number::Number,
            Expression,
        },
        ASTAllocator,
        Parse,
        Span,
    };

    #[test]
    pub fn parses_for() -> anyhow::Result<()> {
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
