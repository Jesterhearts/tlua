use crate::{
    block::Block,
    combinators::{
        parse_separated_list0,
        parse_separated_list1_or,
    },
    identifiers::Ident,
    lexer::Token,
    list::List,
    ASTAllocator,
    ParseError,
    PeekableLexer,
    SyntaxError,
};

#[derive(Debug, PartialEq)]
pub struct FnParams<'chunk> {
    /// Note that LUA 5.4 doesn't distinguish multiple variables during function
    /// evaluation. So a function like `(a, a) return a + a; end` when
    /// called with `(10, 11)` produces `22` in valid lua.
    pub named_params: List<'chunk, Ident>,
    pub varargs: bool,
}

impl<'chunk> FnParams<'chunk> {
    pub(crate) fn parse(
        lexer: &mut PeekableLexer,
        alloc: &'chunk ASTAllocator,
    ) -> Result<Self, ParseError> {
        lexer.expecting_token(Token::LParen)?;

        let named_params = parse_separated_list0(
            lexer,
            alloc,
            |lexer, alloc| Ok(Ident::try_parse(lexer, alloc)),
            |token| *token == Token::Comma,
        )?;

        let varargs = if named_params.is_empty() {
            lexer.next_if_eq(Token::Ellipses).is_some()
        } else {
            lexer.next_if_eq(Token::Comma).map_or(Ok(false), |_| {
                lexer.expecting_token_or(Token::Ellipses, SyntaxError::ExpectedIdentOrVaArgs)?;
                Ok(true)
            })?
        };

        lexer.expecting_token(Token::RParen)?;

        Ok(Self {
            named_params,
            varargs,
        })
    }
}

#[derive(Debug, PartialEq)]
pub struct FnBody<'chunk> {
    pub params: FnParams<'chunk>,
    pub body: Block<'chunk>,
}

impl<'chunk> FnBody<'chunk> {
    pub(crate) fn parse(
        lexer: &mut PeekableLexer,
        alloc: &'chunk ASTAllocator,
    ) -> Result<Self, ParseError> {
        let params = FnParams::parse(lexer, alloc)?;

        let body = Block::parse_with_end(lexer, alloc)?;
        Ok(Self { params, body })
    }
}

#[derive(Debug, PartialEq)]
pub struct FnName<'chunk> {
    pub path: List<'chunk, Ident>,
    pub method: Option<Ident>,
}

impl<'chunk> FnName<'chunk> {
    pub(crate) fn parse(
        lexer: &mut PeekableLexer,
        alloc: &'chunk ASTAllocator,
    ) -> Result<Self, ParseError> {
        let path = parse_separated_list1_or(
            lexer,
            alloc,
            |lexer, alloc| Ident::parse(lexer, alloc).map(Some),
            |token| *token == Token::Period,
            SyntaxError::ExpectedToken(Token::Ident),
        )?;

        let method = if lexer.next_if_eq(Token::Colon).is_some() {
            Some(Ident::parse(lexer, alloc)?)
        } else {
            None
        };

        Ok(Self { path, method })
    }
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;

    use crate::{
        block::{
            retstat::RetStatement,
            Block,
        },
        expressions::{
            function_defs::{
                FnBody,
                FnParams,
            },
            number::Number,
            Expression,
        },
        final_parser,
        identifiers::Ident,
        list::{
            List,
            ListNode,
        },
        ASTAllocator,
        StringTable,
    };

    #[test]
    pub fn parses_empty_params() -> anyhow::Result<()> {
        let src = "()";

        let alloc = ASTAllocator::default();
        let mut strings = StringTable::default();
        let result = final_parser!((src.as_bytes(), &alloc, &mut strings) => FnParams::parse)?;

        assert_eq!(
            result,
            FnParams {
                named_params: Default::default(),
                varargs: false,
            }
        );

        Ok(())
    }

    #[test]
    pub fn parses_params_no_varargs() -> anyhow::Result<()> {
        let src = "(a)";

        let alloc = ASTAllocator::default();
        let mut strings = StringTable::default();
        let result = final_parser!((src.as_bytes(), &alloc, &mut strings) => FnParams::parse)?;

        assert_eq!(
            result,
            FnParams {
                named_params: List::new(&mut ListNode::new(Ident(0))),
                varargs: false,
            }
        );

        Ok(())
    }

    #[test]
    pub fn parses_params_only_varargs() -> anyhow::Result<()> {
        let src = "(...)";

        let alloc = ASTAllocator::default();
        let mut strings = StringTable::default();
        let result = final_parser!((src.as_bytes(), &alloc, &mut strings) => FnParams::parse)?;

        assert_eq!(
            result,
            FnParams {
                named_params: Default::default(),
                varargs: true,
            }
        );

        Ok(())
    }

    #[test]
    pub fn parses_params_trailing_varags() -> anyhow::Result<()> {
        let src = "(a,b, c, ...)";

        let alloc = ASTAllocator::default();
        let mut strings = StringTable::default();
        let result = final_parser!((src.as_bytes(), &alloc, &mut strings) => FnParams::parse)?;

        assert_eq!(
            result,
            FnParams {
                named_params: List::from_slice(&mut [
                    ListNode::new(Ident(0)),
                    ListNode::new(Ident(1)),
                    ListNode::new(Ident(2))
                ]),
                varargs: true,
            }
        );

        Ok(())
    }

    #[test]
    pub fn parses_func_body() -> anyhow::Result<()> {
        let src = "() return 10 end";

        let alloc = ASTAllocator::default();
        let mut strings = StringTable::default();
        let result = final_parser!((src.as_bytes(), &alloc, &mut strings) => FnBody::parse)?;

        assert_eq!(
            result,
            FnBody {
                params: FnParams {
                    named_params: Default::default(),
                    varargs: false,
                },
                body: Block {
                    statements: Default::default(),
                    ret: Some(RetStatement {
                        expressions: List::new(&mut ListNode::new(Expression::Number(
                            Number::Integer(10)
                        )))
                    })
                }
            }
        );
        Ok(())
    }
}
