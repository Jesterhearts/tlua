use crate::{
    expressions::Expression,
    lexer::Token,
    list::List,
    ASTAllocator,
    ParseError,
    PeekableLexer,
};

#[derive(Debug, PartialEq)]
pub struct RetStatement<'chunk> {
    pub expressions: List<'chunk, Expression<'chunk>>,
}

impl<'chunk> RetStatement<'chunk> {
    pub(crate) fn try_parse(
        lexer: &mut PeekableLexer,
        alloc: &'chunk ASTAllocator,
    ) -> Result<Option<Self>, ParseError> {
        if lexer.next_if_eq(Token::KWreturn).is_none() {
            return Ok(None);
        }

        let expressions = Expression::parse_list0(lexer, alloc)?;
        lexer.next_if_eq(Token::Semicolon);

        Ok(Some(Self { expressions }))
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
        let result =
            final_parser!((src.as_bytes(), &alloc, &mut strings) => RetStatement::try_parse)?;

        assert_eq!(
            result,
            Some(RetStatement {
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
        let result =
            final_parser!((src.as_bytes(), &alloc, &mut strings) => RetStatement::try_parse)?;

        assert_eq!(
            result,
            Some(RetStatement {
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
        let result =
            final_parser!((src.as_bytes(), &alloc, &mut strings) => RetStatement::try_parse)?;

        assert_eq!(
            result,
            Some(RetStatement {
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
        let result =
            final_parser!((src.as_bytes(), &alloc, &mut strings) => RetStatement::try_parse)?;

        assert_eq!(
            result,
            Some(RetStatement {
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
        let result =
            final_parser!((src.as_bytes(), &alloc, &mut strings) => RetStatement::try_parse)?;

        assert_eq!(
            result,
            Some(RetStatement {
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
        let result =
            final_parser!((src.as_bytes(), &alloc, &mut strings) => RetStatement::try_parse)?;

        assert_eq!(
            result,
            Some(RetStatement {
                expressions: List::from_slice(&mut [ListNode::new(Expression::Parenthesized(
                    &Expression::Number(Number::Integer(10))
                )),])
            })
        );

        Ok(())
    }
}
