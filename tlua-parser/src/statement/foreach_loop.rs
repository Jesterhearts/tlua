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

use crate::{
    block::Block,
    expressions::{
        expression_list1,
        Expression,
    },
    identifiers::{
        identifier_list1,
        Ident,
    },
    list::List,
    lua_whitespace0,
    lua_whitespace1,
    ASTAllocator,
    ParseResult,
    Span,
};

#[derive(Debug, PartialEq)]
pub struct ForEachLoop<'chunk> {
    pub vars: List<'chunk, Ident>,
    pub expressions: List<'chunk, Expression<'chunk>>,
    pub body: Block<'chunk>,
}

impl<'chunk> ForEachLoop<'chunk> {
    pub(crate) fn parser(
        alloc: &'chunk ASTAllocator,
    ) -> impl for<'src> FnMut(Span<'src>) -> ParseResult<'src, ForEachLoop<'chunk>> {
        |input| {
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
                        terminated(Block::parser(alloc), preceded(lua_whitespace0, tag("end"))),
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
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;

    use super::ForEachLoop;
    use crate::{
        expressions::{
            number::Number,
            Expression,
        },
        final_parser,
        list::{
            List,
            ListNode,
        },
        ASTAllocator,
        Span,
    };

    #[test]
    pub fn parses_foreach() -> anyhow::Result<()> {
        let src = "for a,b,c,d in 1,2,3,4 do end";

        let alloc = ASTAllocator::default();
        let result = final_parser!(Span::new(src.as_bytes()) => ForEachLoop::parser( &alloc))?;

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
