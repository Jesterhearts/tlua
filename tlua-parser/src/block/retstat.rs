use nom::{
    bytes::complete::tag,
    combinator::{
        map,
        opt,
    },
    sequence::delimited,
};

use crate::{
    expressions::{
        build_expression_list0,
        Expression,
    },
    list::List,
    lua_whitespace0,
    ASTAllocator,
    ParseResult,
    Span,
};

#[derive(Debug, PartialEq)]
pub struct RetStatement<'chunk> {
    pub expressions: List<'chunk, Expression<'chunk>>,
}

impl<'chunk> RetStatement<'chunk> {
    pub(crate) fn parser(
        alloc: &'chunk ASTAllocator,
    ) -> impl for<'src> FnMut(Span<'src>) -> ParseResult<'src, RetStatement<'chunk>> {
        |input| {
            map(
                delimited(
                    tag("return"),
                    delimited(
                        lua_whitespace0,
                        build_expression_list0(alloc),
                        lua_whitespace0,
                    ),
                    opt(tag(";")),
                ),
                |expressions| Self { expressions },
            )(input)
        }
    }
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;

    use super::RetStatement;
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
    pub fn parses_empty_ret() -> anyhow::Result<()> {
        let src = "return";

        let alloc = ASTAllocator::default();
        let result = final_parser!(Span::new(src.as_bytes()) => RetStatement::parser(&alloc))?;

        assert_eq!(
            result,
            RetStatement {
                expressions: Default::default()
            }
        );

        Ok(())
    }

    #[test]
    pub fn parses_empty_ret_semicolon() -> anyhow::Result<()> {
        let src = "return;";

        let alloc = ASTAllocator::default();
        let result = final_parser!(Span::new(src.as_bytes()) => RetStatement::parser(&alloc))?;

        assert_eq!(
            result,
            RetStatement {
                expressions: Default::default()
            }
        );

        Ok(())
    }

    #[test]
    pub fn parses_simple_ret() -> anyhow::Result<()> {
        let src = "return 10";

        let alloc = ASTAllocator::default();
        let result = final_parser!(Span::new(src.as_bytes()) => RetStatement::parser(&alloc))?;

        assert_eq!(
            result,
            RetStatement {
                expressions: List::new(&mut ListNode::new(Expression::Number(Number::Integer(10))))
            }
        );

        Ok(())
    }

    #[test]
    pub fn parses_simple_ret_semicolon() -> anyhow::Result<()> {
        let src = "return 10;";

        let alloc = ASTAllocator::default();
        let result = final_parser!(Span::new(src.as_bytes()) => RetStatement::parser(&alloc))?;

        assert_eq!(
            result,
            RetStatement {
                expressions: List::new(&mut ListNode::new(Expression::Number(Number::Integer(10))))
            }
        );

        Ok(())
    }

    #[test]
    pub fn parses_multi_ret() -> anyhow::Result<()> {
        let src = "return 10, 11";

        let alloc = ASTAllocator::default();
        let result = final_parser!(Span::new(src.as_bytes()) => RetStatement::parser(&alloc))?;

        assert_eq!(
            result,
            RetStatement {
                expressions: List::from_slice(&mut [
                    ListNode::new(Expression::Number(Number::Integer(10))),
                    ListNode::new(Expression::Number(Number::Integer(11)))
                ])
            }
        );

        Ok(())
    }
}
