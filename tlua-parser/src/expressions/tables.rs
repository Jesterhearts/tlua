use crate::{
    expressions::Expression,
    identifiers::Ident,
    lexer::{
        SpannedToken,
        Token,
    },
    list::List,
    parse_separated_list0,
    prefix_expression::{
        HeadAtom,
        PrefixExpression,
    },
    token_subset,
    ASTAllocator,
    ParseError,
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

token_subset! {
    FieldToken {
        Token::Ident,
        Token::LBracket,
        Error(SyntaxError::ExpectedTableField)
    }
}

impl<'chunk> Field<'chunk> {
    pub(crate) fn try_parse(
        lexer: &mut PeekableLexer,
        alloc: &'chunk ASTAllocator,
    ) -> Result<Option<Self>, ParseError> {
        let token = if let Some(token) = FieldToken::next(lexer) {
            token
        } else if let Some(expression) = Expression::try_parse(lexer, alloc)? {
            return Ok(Some(Self::Arraylike { expression }));
        } else {
            return Ok(None);
        };

        match token.as_ref() {
            FieldToken::Ident => {
                let name = lexer.strings.add_ident(token.src);
                if lexer.next_if_eq(Token::Equals).is_some() {
                    Ok(Some(Self::Named {
                        name,
                        expression: Expression::parse(lexer, alloc)?,
                    }))
                } else {
                    let expr =
                        PrefixExpression::parse_remaining(HeadAtom::Name(name), lexer, alloc)?;
                    Ok(Some(Self::Arraylike {
                        expression: Expression::from((expr, alloc)),
                    }))
                }
            }
            FieldToken::LBracket => Expression::parse(lexer, alloc).and_then(|index| {
                lexer.expecting_token(Token::RBracket)?;
                lexer.expecting_token(Token::Equals)?;
                Expression::parse(lexer, alloc)
                    .map(|expression| Some(Self::Indexed { index, expression }))
            }),
        }
    }
}

#[derive(Debug, PartialEq)]
pub struct TableConstructor<'chunk> {
    pub fields: List<'chunk, Field<'chunk>>,
}

impl<'chunk> TableConstructor<'chunk> {
    pub(crate) fn try_parse(
        lexer: &mut PeekableLexer,
        alloc: &'chunk ASTAllocator,
    ) -> Result<Option<Self>, ParseError> {
        if lexer.next_if_eq(Token::LBrace).is_none() {
            return Ok(None);
        }

        Self::parse_remaining(lexer, alloc).map(Some)
    }

    pub(crate) fn parse_remaining(
        lexer: &mut PeekableLexer,
        alloc: &'chunk ASTAllocator,
    ) -> Result<Self, ParseError> {
        let match_sep =
            |token: &SpannedToken| matches!(token.as_ref(), Token::Comma | Token::Semicolon);
        let fields = parse_separated_list0(lexer, alloc, Field::try_parse, match_sep)?;
        lexer.next_if(match_sep);

        lexer.expecting_token(Token::RBrace)?;
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
            final_parser!((src.as_bytes(), &alloc, &mut strings) => TableConstructor::try_parse)?;

        assert_eq!(
            result,
            Some(TableConstructor {
                fields: Default::default(),
            })
        );

        Ok(())
    }

    #[test]
    fn arraylike_field_prefix_expr() -> anyhow::Result<()> {
        let src = "{ foo() }";

        let alloc = ASTAllocator::default();
        let mut strings = StringTable::default();
        let result =
            final_parser!((src.as_bytes(), &alloc, &mut strings) => TableConstructor::try_parse)?;

        assert_eq!(
            result,
            Some(TableConstructor {
                fields: List::new(&mut ListNode::new(Field::Arraylike {
                    expression: Expression::FunctionCall(&FnCallPrefixExpression::Call {
                        head: HeadAtom::Name(Ident(0)),
                        args: FunctionAtom::Call(FnArgs::Expressions(Default::default()))
                    })
                })),
            })
        );

        Ok(())
    }

    #[test]
    fn parses_empty_table_semicolon() -> anyhow::Result<()> {
        let src = "{;}";

        let alloc = ASTAllocator::default();
        let mut strings = StringTable::default();
        let result =
            final_parser!((src.as_bytes(), &alloc, &mut strings) => TableConstructor::try_parse)?;

        assert_eq!(
            result,
            Some(TableConstructor {
                fields: Default::default(),
            })
        );
        Ok(())
    }

    #[test]
    fn parses_arraylike_field() -> anyhow::Result<()> {
        let src = "10";

        let alloc = ASTAllocator::default();
        let mut strings = StringTable::default();
        let result = final_parser!((src.as_bytes(), &alloc, &mut strings) => Field::try_parse)?;

        assert_eq!(
            result,
            Some(Field::Arraylike {
                expression: Expression::Number(Number::Integer(10))
            })
        );

        Ok(())
    }

    #[test]
    fn parses_named_field() -> anyhow::Result<()> {
        let src = "a = 10";

        let alloc = ASTAllocator::default();
        let mut strings = StringTable::default();
        let result = final_parser!((src.as_bytes(), &alloc, &mut strings) => Field::try_parse)?;

        assert_eq!(
            result,
            Some(Field::Named {
                name: Ident(0),
                expression: Expression::Number(Number::Integer(10))
            })
        );

        Ok(())
    }

    #[test]
    fn parses_index_field() -> anyhow::Result<()> {
        let src = "[11] = 10";

        let alloc = ASTAllocator::default();
        let mut strings = StringTable::default();
        let result = final_parser!((src.as_bytes(), &alloc, &mut strings) => Field::try_parse)?;

        assert_eq!(
            result,
            Some(Field::Indexed {
                index: Expression::Number(Number::Integer(11)),
                expression: Expression::Number(Number::Integer(10))
            })
        );

        Ok(())
    }

    #[test]
    fn parses_parses_table_mixed() -> anyhow::Result<()> {
        let src = "{10, [11] = 12, a = 13; 14}";

        let alloc = ASTAllocator::default();
        let mut strings = StringTable::default();
        let result =
            final_parser!((src.as_bytes(), &alloc, &mut strings) => TableConstructor::try_parse)?;

        assert_eq!(
            result,
            Some(TableConstructor {
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
            })
        );

        Ok(())
    }
}
