use crate::{
    expressions::{
        strings::ConstantString,
        tables::TableConstructor,
        Expression,
    },
    lexer::Token,
    list::List,
    ASTAllocator,
    ParseError,
    ParseErrorExt,
    PeekableLexer,
    SyntaxError,
};

#[derive(Debug, PartialEq)]
pub enum FnArgs<'chunk> {
    Expressions(List<'chunk, Expression<'chunk>>),
    TableConstructor(TableConstructor<'chunk>),
    String(ConstantString),
}

impl<'chunk> FnArgs<'chunk> {
    pub(crate) fn parse(
        lexer: &mut PeekableLexer,
        alloc: &'chunk ASTAllocator,
    ) -> Result<Self, ParseError> {
        ConstantString::parse(lexer, alloc)
            .map(Self::String)
            .recover_with(|| TableConstructor::parse(lexer, alloc).map(Self::TableConstructor))
            .recover_with(|| {
                lexer.expecting_token(Token::LParen)?;

                let exprs = Expression::parse_list0(lexer, alloc)?;

                lexer.expecting_token(Token::RParen).mark_unrecoverable()?;

                Ok(Self::Expressions(exprs))
            })
            .ok_or_else(|| ParseError::recoverable_from_here(lexer, SyntaxError::ExpectedFnArgs))
    }
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;

    use super::FnArgs;
    use crate::{
        expressions::{
            strings::ConstantString,
            tables::TableConstructor,
            Expression,
            Nil,
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
    pub fn parses_empty_args() -> anyhow::Result<()> {
        let src = "()";

        let alloc = ASTAllocator::default();
        let mut strings = StringTable::default();
        let result = final_parser!((src.as_bytes(), &alloc, &mut strings) => FnArgs::parse)?;

        assert_eq!(result, FnArgs::Expressions(Default::default()));

        Ok(())
    }

    #[test]
    pub fn parses_parenthetical_args() -> anyhow::Result<()> {
        let src = "(nil, nil, nil)";

        let alloc = ASTAllocator::default();
        let mut strings = StringTable::default();
        let result = final_parser!((src.as_bytes(), &alloc, &mut strings) => FnArgs::parse)?;

        assert_eq!(
            result,
            FnArgs::Expressions(List::from_slice(&mut [
                ListNode::new(Expression::Nil(Nil)),
                ListNode::new(Expression::Nil(Nil)),
                ListNode::new(Expression::Nil(Nil))
            ]))
        );

        Ok(())
    }

    #[test]
    pub fn parses_table_args() -> anyhow::Result<()> {
        let src = "{}";

        let alloc = ASTAllocator::default();
        let mut strings = StringTable::default();
        let result = final_parser!((src.as_bytes(), &alloc, &mut strings) => FnArgs::parse)?;

        assert_eq!(
            result,
            FnArgs::TableConstructor(TableConstructor {
                fields: Default::default(),
            })
        );

        Ok(())
    }

    #[test]
    pub fn parses_string_args() -> anyhow::Result<()> {
        let src = "\"arg\"";

        let alloc = ASTAllocator::default();
        let mut strings = StringTable::default();
        let result = final_parser!((src.as_bytes(), &alloc, &mut strings) => FnArgs::parse)?;

        assert_eq!(result, FnArgs::String(ConstantString(0)));

        Ok(())
    }
}
