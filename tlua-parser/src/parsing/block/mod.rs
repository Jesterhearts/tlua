use nom::{
    combinator::opt,
    sequence::terminated,
};
use tracing::instrument;

use crate::{
    ast::{
        block::{
            retstat::RetStatement,
            Block,
        },
        statement::Statement,
    },
    list::List,
    parsing::{
        lua_whitespace0,
        ASTAllocator,
        Parse,
        ParseResult,
        Span,
    },
};

pub mod retstat;

impl<'chunk> Parse<'chunk> for Block<'chunk> {
    #[instrument(level = "trace", name = "block", skip(input, alloc))]
    fn parse<'src>(mut input: Span<'src>, alloc: &'chunk ASTAllocator) -> ParseResult<'src, Self> {
        let (remain, stat) = opt(terminated(
            |input| Statement::parse(input, alloc),
            lua_whitespace0,
        ))(input)?;
        input = remain;

        let mut statements = List::default();

        let mut current = if let Some(head) = stat {
            statements.cursor_mut().alloc_insert_advance(alloc, head)
        } else {
            let (remain, ret) = opt(|input| RetStatement::parse(input, alloc))(input)?;

            return Ok((remain, Block { ret, statements }));
        };

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
    use nom_supreme::final_parser::final_parser;
    use pretty_assertions::assert_eq;

    use super::Block;
    use crate::{
        ast::{
            block::retstat::RetStatement,
            expressions::{
                number::Number,
                Expression,
            },
        },
        list::{
            List,
            ListNode,
        },
        parsing::{
            ASTAllocator,
            InternalLuaParseError,
            LuaParseError,
            Parse,
            Span,
        },
    };

    #[test]
    pub fn parses_empty_body() -> anyhow::Result<()> {
        let src = "";

        let alloc = ASTAllocator::default();
        let result = final_parser(|input| Block::parse(input, &alloc))(Span::new(src.as_bytes()))
            .map_err(|e: InternalLuaParseError| e.map_locations(LuaParseError::from))?;

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
        let result = final_parser(|input| Block::parse(input, &alloc))(Span::new(src.as_bytes()))
            .map_err(|e: InternalLuaParseError| e.map_locations(LuaParseError::from))?;

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
