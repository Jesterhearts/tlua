use nom::{
    combinator::opt,
    sequence::terminated,
};

use crate::{
    list::List,
    lua_whitespace0,
    statement::Statement,
    ASTAllocator,
    Parse,
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

impl<'chunk> Parse<'chunk> for Block<'chunk> {
    fn parse<'src>(mut input: Span<'src>, alloc: &'chunk ASTAllocator) -> ParseResult<'src, Self> {
        let mut statements = List::default();
        let mut current = statements.cursor_mut();

        loop {
            let (remain, maybe_next) = opt(terminated(
                |input| Statement::parse(input, alloc),
                lua_whitespace0,
            ))(input)?;
            input = remain;

            current = if let Some(next) = maybe_next {
                current.alloc_insert_advance(alloc, next)
            } else {
                break;
            };
        }

        let (remain, ret) = opt(|input| RetStatement::parse(input, alloc))(input)?;

        Ok((remain, Block { statements, ret }))
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
        Parse,
        Span,
    };

    #[test]
    pub fn parses_empty_body() -> anyhow::Result<()> {
        let src = "";

        let alloc = ASTAllocator::default();
        let result =
            final_parser!(Span::new(src.as_bytes()) => |input| Block::parse(input, &alloc))?;

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
        let result =
            final_parser!(Span::new(src.as_bytes()) => |input| Block::parse(input, &alloc))?;

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
