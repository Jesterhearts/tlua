use crate::{
    lexer::Token,
    list::List,
    parse_list0,
    statement::Statement,
    ASTAllocator,
    ParseError,
    ParseErrorExt,
    PeekableLexer,
    SyntaxError,
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
        let statements = parse_list0(lexer, alloc, Statement::parse)?;
        let ret = RetStatement::parse(lexer, alloc).recover()?;

        Ok(Self { statements, ret })
    }

    pub(crate) fn parse_do(
        lexer: &mut PeekableLexer,
        alloc: &'chunk ASTAllocator,
    ) -> Result<Self, ParseError> {
        lexer.next_if_eq(Token::KWdo).ok_or_else(|| {
            ParseError::recoverable_from_here(lexer, SyntaxError::ExpectedToken(Token::KWdo))
        })?;
        let body = Self::parse(lexer, alloc).mark_unrecoverable()?;
        lexer.next_if_eq(Token::KWend).ok_or_else(|| {
            ParseError::unrecoverable_from_here(lexer, SyntaxError::ExpectedToken(Token::KWend))
        })?;

        Ok(body)
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
        let src = "";

        let alloc = ASTAllocator::default();
        let mut strings = StringTable::default();
        let result = final_parser!((src.as_bytes(), &alloc, &mut strings) => Block::parse)?;

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
