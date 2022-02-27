use crate::{
    expressions::Expression,
    identifiers::Ident,
    lexer::{
        SpannedToken,
        Token,
    },
    list::List,
    parse_separated_list0,
    ASTAllocator,
    ParseError,
    ParseErrorExt,
    PeekableLexer,
    SyntaxError,
};

/// Field values for a field list ordered in ascending order of precedence.
///
/// If you have an expression like:
/// ```lua
/// {10, 11, [1] = 13}
/// -- alternatively
/// {[1] = 13, 10, 11}
/// ```
/// Your final table will always contain `{10, 11}` as of Lua 5.4
#[derive(Debug, PartialEq)]
pub enum Field<'chunk> {
    /// `{ 'Name' ='Exp' }`
    Named {
        name: Ident,
        expression: Expression<'chunk>,
    },
    /// `{ ['Exp'] ='Exp' }`
    Indexed {
        index: Expression<'chunk>,
        expression: Expression<'chunk>,
    },
    /// `{ 'Exp' }`
    ///
    /// `{ 'Exp1', 'Exp2' } ` behaves like `['Exp1', 'Exp2']` with 1-based
    /// indexing.
    Arraylike { expression: Expression<'chunk> },
}

#[derive(Debug, PartialEq)]
pub struct TableConstructor<'chunk> {
    pub fields: List<'chunk, Field<'chunk>>,
}

impl<'chunk> Field<'chunk> {
    pub(crate) fn parse(
        lexer: &mut PeekableLexer,
        alloc: &'chunk ASTAllocator,
    ) -> Result<Self, ParseError> {
        lexer
            .peek()
            .filter(|token| *token == Token::Ident)
            .ok_or_else(|| {
                ParseError::recoverable_from_here(lexer, SyntaxError::ExpectedToken(Token::Ident))
            })
            .and_then(|token| {
                Ident::parse(lexer, alloc).and_then(|name| {
                    lexer.next_if_eq(Token::Equals).ok_or_else(|| {
                        lexer.reset(token);
                        ParseError::recoverable_from_here(
                            lexer,
                            SyntaxError::ExpectedToken(Token::Equals),
                        )
                    })?;

                    let expression = Expression::parse(lexer, alloc)?;
                    Ok(Self::Named { name, expression })
                })
            })
            .recover_with(|| {
                lexer.next_if_eq(Token::LBracket).ok_or_else(|| {
                    ParseError::recoverable_from_here(
                        lexer,
                        SyntaxError::ExpectedToken(Token::LBracket),
                    )
                })?;
                let index = Expression::parse(lexer, alloc)?;

                lexer.next_if_eq(Token::RBracket).ok_or_else(|| {
                    ParseError::unrecoverable_from_here(
                        lexer,
                        SyntaxError::ExpectedToken(Token::RBracket),
                    )
                })?;

                lexer.next_if_eq(Token::Equals).ok_or_else(|| {
                    ParseError::unrecoverable_from_here(
                        lexer,
                        SyntaxError::ExpectedToken(Token::Equals),
                    )
                })?;

                let expression = Expression::parse(lexer, alloc)?;

                Ok(Self::Indexed { index, expression })
            })
            .recover_with(|| {
                Expression::parse(lexer, alloc).map(|expression| Self::Arraylike { expression })
            })
            .ok_or_else(|| {
                ParseError::recoverable_from_here(lexer, SyntaxError::ExpectedTableField)
            })
    }
}

impl<'chunk> TableConstructor<'chunk> {
    pub(crate) fn parse(
        lexer: &mut PeekableLexer,
        alloc: &'chunk ASTAllocator,
    ) -> Result<Self, ParseError> {
        lexer.next_if_eq(Token::LBrace).ok_or_else(|| {
            ParseError::recoverable_from_here(lexer, SyntaxError::ExpectedToken(Token::LBrace))
        })?;

        let match_sep =
            |token: &SpannedToken| matches!(token.as_ref(), Token::Comma | Token::Semicolon);
        let fields = parse_separated_list0(lexer, alloc, Field::parse, match_sep)?;
        lexer.next_if(match_sep);

        lexer.next_if_eq(Token::RBrace).ok_or_else(|| {
            ParseError::unrecoverable_from_here(lexer, SyntaxError::ExpectedToken(Token::RBrace))
        })?;

        Ok(Self { fields })
    }
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;

    use super::TableConstructor;
    use crate::{
        expressions::{
            number::Number,
            tables::Field,
            Expression,
        },
        final_parser,
        identifiers::Ident,
        list::{
            List,
            ListNode,
        },
        prefix_expression::{
            function_calls::FnArgs,
            FnCallPrefixExpression,
            FunctionAtom,
            HeadAtom,
        },
        ASTAllocator,
        StringTable,
    };

    #[test]
    fn parses_empty_table() -> anyhow::Result<()> {
        let src = "{}";

        let alloc = ASTAllocator::default();
        let mut strings = StringTable::default();
        let result =
            final_parser!((src.as_bytes(), &alloc, &mut strings) => TableConstructor::parse)?;

        assert_eq!(
            result,
            TableConstructor {
                fields: Default::default(),
            }
        );

        Ok(())
    }

    #[test]
    fn arraylike_field_prefix_expr() -> anyhow::Result<()> {
        let src = "{ foo() }";

        let alloc = ASTAllocator::default();
        let mut strings = StringTable::default();
        let result =
            final_parser!((src.as_bytes(), &alloc, &mut strings) => TableConstructor::parse)?;

        assert_eq!(
            result,
            TableConstructor {
                fields: List::new(&mut ListNode::new(Field::Arraylike {
                    expression: Expression::FunctionCall(&FnCallPrefixExpression::Call {
                        head: HeadAtom::Name(Ident(0)),
                        args: FunctionAtom::Call(FnArgs::Expressions(Default::default()))
                    })
                })),
            }
        );

        Ok(())
    }

    #[test]
    fn parses_empty_table_semicolon() -> anyhow::Result<()> {
        let src = "{;}";

        let alloc = ASTAllocator::default();
        let mut strings = StringTable::default();
        let result =
            final_parser!((src.as_bytes(), &alloc, &mut strings) => TableConstructor::parse)?;

        assert_eq!(
            result,
            TableConstructor {
                fields: Default::default(),
            }
        );
        Ok(())
    }

    #[test]
    fn parses_arraylike_field() -> anyhow::Result<()> {
        let src = "10";

        let alloc = ASTAllocator::default();
        let mut strings = StringTable::default();
        let result = final_parser!((src.as_bytes(), &alloc, &mut strings) => Field::parse)?;

        assert_eq!(
            result,
            Field::Arraylike {
                expression: Expression::Number(Number::Integer(10))
            }
        );

        Ok(())
    }

    #[test]
    fn parses_named_field() -> anyhow::Result<()> {
        let src = "a = 10";

        let alloc = ASTAllocator::default();
        let mut strings = StringTable::default();
        let result = final_parser!((src.as_bytes(), &alloc, &mut strings) => Field::parse)?;

        assert_eq!(
            result,
            Field::Named {
                name: Ident(0),
                expression: Expression::Number(Number::Integer(10))
            }
        );

        Ok(())
    }

    #[test]
    fn parses_index_field() -> anyhow::Result<()> {
        let src = "[11] = 10";

        let alloc = ASTAllocator::default();
        let mut strings = StringTable::default();
        let result = final_parser!((src.as_bytes(), &alloc, &mut strings) => Field::parse)?;

        assert_eq!(
            result,
            Field::Indexed {
                index: Expression::Number(Number::Integer(11)),
                expression: Expression::Number(Number::Integer(10))
            }
        );

        Ok(())
    }

    #[test]
    fn parses_parses_table_mixed() -> anyhow::Result<()> {
        let src = "{10, [11] = 12, a = 13; 14}";

        let alloc = ASTAllocator::default();
        let mut strings = StringTable::default();
        let result =
            final_parser!((src.as_bytes(), &alloc, &mut strings) => TableConstructor::parse)?;

        assert_eq!(
            result,
            TableConstructor {
                fields: List::from_slice(&mut [
                    ListNode::new(Field::Arraylike {
                        expression: Expression::Number(Number::Integer(10))
                    }),
                    ListNode::new(Field::Indexed {
                        index: Expression::Number(Number::Integer(11)),
                        expression: Expression::Number(Number::Integer(12)),
                    }),
                    ListNode::new(Field::Named {
                        name: Ident(0),
                        expression: Expression::Number(Number::Integer(13)),
                    }),
                    ListNode::new(Field::Arraylike {
                        expression: Expression::Number(Number::Integer(14))
                    }),
                ]),
            }
        );

        Ok(())
    }
}
