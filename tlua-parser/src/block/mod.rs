use nom::{
    branch::alt,
    combinator::{
        eof,
        map,
        opt,
    },
    sequence::{
        pair,
        terminated,
    },
};

use crate::{
    build_list0,
    build_list1,
    list::List,
    lua_whitespace0,
    statement::Statement,
    ASTAllocator,
    ParseResult,
    Span,
};

pub mod retstat;
use self::retstat::RetStatement;

#[derive(Debug, Default, PartialEq)]
pub struct Block<'chunk> {
    pub statements: List<'chunk, Statement<'chunk>>,
    pub ret: Option<RetStatement<'chunk>>,
}

impl<'chunk> Block<'chunk> {
    pub(crate) fn parser(
        alloc: &'chunk ASTAllocator,
    ) -> impl for<'src> FnMut(Span<'src>) -> ParseResult<'src, Block<'chunk>> {
        |input| {
            map(
                pair(
                    build_list0(alloc, terminated(Statement::parser(alloc), lua_whitespace0)),
                    opt(RetStatement::parser(alloc)),
                ),
                |(statements, ret)| Block { statements, ret },
            )(input)
        }
    }

    pub(crate) fn main_parser(
        alloc: &'chunk ASTAllocator,
    ) -> impl for<'src> FnMut(Span<'src>) -> ParseResult<'src, Block<'chunk>> {
        |input| {
            alt((
                map(eof, |_| Self::default()),
                map(
                    pair(
                        build_list1(alloc, terminated(Statement::parser(alloc), lua_whitespace0)),
                        opt(RetStatement::parser(alloc)),
                    ),
                    |(statements, ret)| Block { statements, ret },
                ),
                map(RetStatement::parser(alloc), |ret| Block {
                    statements: Default::default(),
                    ret: Some(ret),
                }),
            ))(input)
        }
    }
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;

    use super::Block;
    use crate::{
        block::retstat::RetStatement,
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
    pub fn parses_empty_body() -> anyhow::Result<()> {
        let src = "";

        let alloc = ASTAllocator::default();
        let result = final_parser!(Span::new(src.as_bytes()) => Block::parser(&alloc))?;

        assert_eq!(
            result,
            Block {
                statements: Default::default(),
                ret: None
            }
        );

        Ok(())
    }

    #[test]
    pub fn parses_only_return() -> anyhow::Result<()> {
        let src = "return 10";

        let alloc = ASTAllocator::default();
        let result = final_parser!(Span::new(src.as_bytes()) => Block::parser( &alloc))?;

        assert_eq!(
            result,
            Block {
                statements: Default::default(),
                ret: Some(RetStatement {
                    expressions: List::new(&mut ListNode::new(Expression::Number(
                        Number::Integer(10)
                    )))
                })
            }
        );

        Ok(())
    }
}
