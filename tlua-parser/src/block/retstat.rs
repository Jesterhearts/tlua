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
        expression_list1,
        Expression,
    },
    list::List,
    lua_whitespace0,
    ASTAllocator,
    Parse,
    ParseResult,
    Span,
};

#[derive(Debug, PartialEq)]
pub struct RetStatement<'chunk> {
    pub expressions: List<'chunk, Expression<'chunk>>,
}

impl<'chunk> Parse<'chunk> for RetStatement<'chunk> {
    fn parse<'src>(input: Span<'src>, alloc: &'chunk ASTAllocator) -> ParseResult<'src, Self> {
        map(
            delimited(
                tag("return"),
                delimited(
                    lua_whitespace0,
                    opt(|input| expression_list1(input, alloc)),
                    lua_whitespace0,
                ),
                opt(tag(";")),
            ),
            |expressions| Self {
                expressions: expressions.unwrap_or_default(),
            },
        )(input)
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
        list::{
            List,
            ListNode,
        },
        ASTAllocator,
        Parse,
        Span,
    };

    #[test]
    pub fn parses_empty_ret() -> anyhow::Result<()> {
        let src = "return";

        let alloc = ASTAllocator::default();
        let (remain, result) = RetStatement::parse(Span::new(src.as_bytes()), &alloc)?;

        assert_eq!(std::str::from_utf8(*remain)?, "");
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
        let (remain, result) = RetStatement::parse(Span::new(src.as_bytes()), &alloc)?;

        assert_eq!(std::str::from_utf8(*remain)?, "");
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
        let (remain, result) = RetStatement::parse(Span::new(src.as_bytes()), &alloc)?;

        assert_eq!(std::str::from_utf8(*remain)?, "");
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
        let (remain, result) = RetStatement::parse(Span::new(src.as_bytes()), &alloc)?;

        assert_eq!(std::str::from_utf8(*remain)?, "");
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
        let (remain, result) = RetStatement::parse(Span::new(src.as_bytes()), &alloc)?;

        assert_eq!(std::str::from_utf8(*remain)?, "");
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
