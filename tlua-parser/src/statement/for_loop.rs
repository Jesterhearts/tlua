use nom::{
    character::complete::char as token,
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
    expressions::Expression,
    identifiers::{
        keyword,
        Ident,
    },
    lua_whitespace0,
    lua_whitespace1,
    ASTAllocator,
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

impl<'chunk> ForLoop<'chunk> {
    pub(crate) fn parser(
        alloc: &'chunk ASTAllocator,
    ) -> impl for<'src> FnMut(Span<'src>) -> ParseResult<'src, ForLoop<'chunk>> {
        |input| {
            preceded(
                pair(keyword("for"), lua_whitespace1),
                map(
                    pair(
                        terminated(
                            Ident::parser(alloc),
                            delimited(lua_whitespace0, token('='), lua_whitespace0),
                        ),
                        cut(tuple((
                            terminated(
                                Expression::parser(alloc),
                                delimited(lua_whitespace0, token(','), lua_whitespace0),
                            ),
                            Expression::parser(alloc),
                            opt(preceded(
                                delimited(lua_whitespace0, token(','), lua_whitespace0),
                                Expression::parser(alloc),
                            )),
                            delimited(
                                delimited(lua_whitespace0, keyword("do"), lua_whitespace1),
                                Block::parser(alloc),
                                preceded(lua_whitespace0, keyword("end")),
                            ),
                        ))),
                    ),
                    |(var, (init, condition, increment, body))| Self {
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
        final_parser,
        ASTAllocator,
        Span,
    };

    #[test]
    pub fn parses_for() -> anyhow::Result<()> {
        let src = "for a = 0, 10 do end";

        let alloc = ASTAllocator::default();
        let result = final_parser!(Span::new(src.as_bytes())=> ForLoop::parser( &alloc))?;

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
