use crate::{
    expressions::Expression,
    lexer::Token,
    list::List,
    ASTAllocator,
    ParseError,
    PeekableLexer,
    SyntaxError,
};

#[derive(Debug, PartialEq)]
pub struct RetStatement<'chunk> {
    pub expressions: List<'chunk, Expression<'chunk>>,
}

impl<'chunk> RetStatement<'chunk> {
    pub(crate) fn parse(
        lexer: &mut PeekableLexer,
        alloc: &'chunk ASTAllocator,
    ) -> Result<Self, ParseError> {
        lexer.next_if_eq(Token::KWreturn).ok_or_else(|| {
            ParseError::recoverable_from_here(lexer, SyntaxError::ExpectedToken(Token::KWreturn))
        })?;

        let expressions = Expression::parse_list0(lexer, alloc)?;
        lexer.next_if_eq(Token::Semicolon);

        Ok(Self { expressions })
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
        StringTable,
    };

    #[test]
    pub fn parses_empty_ret() -> anyhow::Result<()> {
        let src = "return";

        let alloc = ASTAllocator::default();
        let mut strings = StringTable::default();
        let result = final_parser!((src.as_bytes(), &alloc, &mut strings) => RetStatement::parse)?;

        assert_eq!(
            result,
            (RetStatement {
                expressions: Default::default()
            })
        );

        Ok(())
    }

    #[test]
    pub fn parses_empty_ret_semicolon() -> anyhow::Result<()> {
        let src = "return;";

        let alloc = ASTAllocator::default();
        let mut strings = StringTable::default();
        let result = final_parser!((src.as_bytes(), &alloc, &mut strings) => RetStatement::parse)?;

        assert_eq!(
            result,
            (RetStatement {
                expressions: Default::default()
            })
        );

        Ok(())
    }

    #[test]
    pub fn parses_simple_ret() -> anyhow::Result<()> {
        let src = "return 10";

        let alloc = ASTAllocator::default();
        let mut strings = StringTable::default();
        let result = final_parser!((src.as_bytes(), &alloc, &mut strings) => RetStatement::parse)?;

        assert_eq!(
            result,
            (RetStatement {
                expressions: List::new(&mut ListNode::new(Expression::Number(Number::Integer(10))))
            })
        );

        Ok(())
    }

    #[test]
    pub fn parses_simple_ret_semicolon() -> anyhow::Result<()> {
        let src = "return 10;";

        let alloc = ASTAllocator::default();
        let mut strings = StringTable::default();
        let result = final_parser!((src.as_bytes(), &alloc, &mut strings) => RetStatement::parse)?;

        assert_eq!(
            result,
            (RetStatement {
                expressions: List::new(&mut ListNode::new(Expression::Number(Number::Integer(10))))
            })
        );

        Ok(())
    }

    #[test]
    pub fn parses_multi_ret() -> anyhow::Result<()> {
        let src = "return 10, 11";

        let alloc = ASTAllocator::default();
        let mut strings = StringTable::default();
        let result = final_parser!((src.as_bytes(), &alloc, &mut strings) => RetStatement::parse)?;

        assert_eq!(
            result,
            (RetStatement {
                expressions: List::from_slice(&mut [
                    ListNode::new(Expression::Number(Number::Integer(10))),
                    ListNode::new(Expression::Number(Number::Integer(11)))
                ])
            })
        );

        Ok(())
    }

    #[test]
    pub fn parses_paren() -> anyhow::Result<()> {
        let src = "return(10)";

        let alloc = ASTAllocator::default();
        let mut strings = StringTable::default();
        let result = final_parser!((src.as_bytes(), &alloc, &mut strings) => RetStatement::parse)?;

        assert_eq!(
            result,
            (RetStatement {
                expressions: List::from_slice(&mut [ListNode::new(Expression::Parenthesized(
                    &Expression::Number(Number::Integer(10))
                )),])
            })
        );

        Ok(())
    }
}
