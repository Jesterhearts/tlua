use nom::{
    bytes::complete::tag,
    combinator::map,
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
        statement::foreach_loop::ForEachLoop,
    },
    parsing::{
        expressions::expression_list1,
        identifiers::identifier_list1,
        lua_whitespace0,
        lua_whitespace1,
        ASTAllocator,
        Parse,
        ParseResult,
        Span,
    },
};

impl<'chunk> ForEachLoop<'chunk> {
    #[instrument(level = "trace", name = "foreach", skip(input, alloc))]
    pub fn parse<'src>(input: Span<'src>, alloc: &'chunk ASTAllocator) -> ParseResult<'src, Self> {
        preceded(
            pair(tag("for"), lua_whitespace1),
            map(
                tuple((
                    terminated(
                        |input| identifier_list1(input, alloc),
                        delimited(lua_whitespace0, tag("in"), lua_whitespace1),
                    ),
                    terminated(
                        |input| expression_list1(input, alloc),
                        delimited(lua_whitespace0, tag("do"), lua_whitespace1),
                    ),
                    terminated(
                        |input| Block::parse(input, alloc),
                        preceded(lua_whitespace0, tag("end")),
                    ),
                )),
                |(vars, expressions, body)| Self {
                    vars,
                    expressions,
                    body,
                },
            ),
        )(input)
    }
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;

    use super::ForEachLoop;
    use crate::{
        ast::expressions::{
            number::Number,
            Expression,
        },
        list::{
            List,
            ListNode,
        },
        parsing::{
            ASTAllocator,
            Span,
        },
    };

    #[test]
    pub fn parses_foreach() -> anyhow::Result<()> {
        let src = "for a,b,c,d in 1,2,3,4 do end";

        let alloc = ASTAllocator::default();
        let (remain, result) = ForEachLoop::parse(Span::new(src.as_bytes()), &alloc)?;

        assert_eq!(std::str::from_utf8(*remain)?, "");
        assert_eq!(
            result,
            ForEachLoop {
                vars: List::from_slice(&mut [
                    ListNode::new("a".into()),
                    ListNode::new("b".into()),
                    ListNode::new("c".into()),
                    ListNode::new("d".into()),
                ]),
                expressions: List::from_slice(&mut [
                    ListNode::new(Expression::Number(Number::Integer(1))),
                    ListNode::new(Expression::Number(Number::Integer(2))),
                    ListNode::new(Expression::Number(Number::Integer(3))),
                    ListNode::new(Expression::Number(Number::Integer(4))),
                ]),
                body: Default::default()
            }
        );

        Ok(())
    }
}
