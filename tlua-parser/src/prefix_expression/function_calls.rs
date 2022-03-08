use crate::{
    expressions::{
        strings::ConstantString,
        tables::TableConstructor,
        Expression,
    },
    lexer::Token,
    list::List,
    token_subset,
    ASTAllocator,
    ParseError,
    PeekableLexer,
    SyntaxError,
};

#[derive(Debug, PartialEq)]
pub enum FnArgs<'chunk> {
    Expressions(List<'chunk, Expression<'chunk>>),
    TableConstructor(TableConstructor<'chunk>),
    String(ConstantString),
}

token_subset! {
    ArgToken {
        Token::LBrace,
        Token::LParen,
        Error(SyntaxError::ExpectedFnArgs)
    }
}

impl<'chunk> FnArgs<'chunk> {
    pub(crate) fn parse(
        lexer: &mut PeekableLexer,
        alloc: &'chunk ASTAllocator,
    ) -> Result<Self, ParseError> {
        Self::try_parse(lexer, alloc).and_then(|args| {
            args.ok_or_else(|| ParseError::from_here(lexer, SyntaxError::ExpectedFnArgs))
        })
    }

    pub(crate) fn try_parse(
        lexer: &mut PeekableLexer,
        alloc: &'chunk ASTAllocator,
    ) -> Result<Option<Self>, ParseError> {
        let token = if let Some(token) = ArgToken::next(lexer) {
            token
        } else if let Some(string) = ConstantString::try_parse(lexer, alloc)? {
            return Ok(Some(Self::String(string)));
        } else {
            return Ok(None);
        };

        match token.as_ref() {
            ArgToken::LBrace => TableConstructor::parse_remaining(lexer, alloc)
                .map(Self::TableConstructor)
                .map(Some),
            ArgToken::LParen => Expression::parse_list0(lexer, alloc).and_then(|expr| {
                lexer
                    .expecting_token(Token::RParen)
                    .map(|_| Some(Self::Expressions(expr)))
            }),
        }
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
