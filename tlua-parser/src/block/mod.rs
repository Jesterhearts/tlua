use crate::{
    lexer::Token,
    list::List,
    parse_list1,
    statement::Statement,
    ASTAllocator,
    ParseError,
    ParseErrorExt,
    PeekableLexer,
};

pub mod retstat;
use self::retstat::RetStatement;

#[derive(Debug, Default, PartialEq)]
pub struct Block<'chunk> {
    pub statements: List<'chunk, Statement<'chunk>>,
    pub ret: Option<RetStatement<'chunk>>,
}

impl<'chunk> Block<'chunk> {
    pub(crate) fn parse(
        lexer: &mut PeekableLexer,
        alloc: &'chunk ASTAllocator,
    ) -> Result<Self, ParseError> {
        parse_list1(lexer, alloc, Statement::parse)
            .and_then(|statements| {
                RetStatement::parse(lexer, alloc)
                    .recover()
                    .map(|ret| Self { statements, ret })
            })
            .recover_with(|| {
                RetStatement::parse(lexer, alloc).map(|ret| Self {
                    statements: Default::default(),
                    ret: Some(ret),
                })
            })
    }

    pub(crate) fn parse_do(
        lexer: &mut PeekableLexer,
        alloc: &'chunk ASTAllocator,
    ) -> Result<Self, ParseError> {
        lexer.expecting_token(Token::KWdo)?;

        Self::parse_with_end(lexer, alloc)
    }

    pub(crate) fn parse_with_end(
        lexer: &mut PeekableLexer,
        alloc: &'chunk ASTAllocator,
    ) -> Result<Self, ParseError> {
        Self::parse(lexer, alloc)
            .chain_or_recover_with(|| lexer.expecting_token(Token::KWend).mark_unrecoverable())
            .map(|(block, _)| block.unwrap_or_default())
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
        StringTable,
    };

    #[test]
    pub fn parses_empty_body() -> anyhow::Result<()> {
        let src = "do end";

        let alloc = ASTAllocator::default();
        let mut strings = StringTable::default();
        let result = final_parser!((src.as_bytes(), &alloc, &mut strings) => Block::parse_do)?;

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
        let mut strings = StringTable::default();
        let result = final_parser!((src.as_bytes(), &alloc, &mut strings) => Block::parse)?;

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
