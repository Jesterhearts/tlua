use nom::{
    branch::alt,
    bytes::complete::tag,
    character::complete::one_of,
    combinator::{
        map,
        opt,
    },
    sequence::{
        delimited,
        pair,
        preceded,
    },
};

use crate::{
    expressions::Expression,
    identifiers::Ident,
    list::List,
    lua_whitespace0,
    ASTAllocator,
    ParseResult,
    Span,
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
    pub(crate) fn parser(
        alloc: &'chunk ASTAllocator,
    ) -> impl for<'src> FnMut(Span<'src>) -> ParseResult<'src, Field<'chunk>> {
        |input| {
            alt((
                map(
                    pair(
                        delimited(
                            pair(tag("["), lua_whitespace0),
                            Expression::parser(alloc),
                            pair(lua_whitespace0, tag("]")),
                        ),
                        preceded(
                            delimited(lua_whitespace0, tag("="), lua_whitespace0),
                            Expression::parser(alloc),
                        ),
                    ),
                    |(index, expression)| Self::Indexed { index, expression },
                ),
                map(
                    pair(
                        Ident::parser(alloc),
                        preceded(
                            delimited(lua_whitespace0, tag("="), lua_whitespace0),
                            Expression::parser(alloc),
                        ),
                    ),
                    |(name, expression)| Self::Named { name, expression },
                ),
                map(
                    preceded(lua_whitespace0, Expression::parser(alloc)),
                    |expression| Self::Arraylike { expression },
                ),
            ))(input)
        }
    }
}

impl<'chunk> TableConstructor<'chunk> {
    pub(crate) fn parser(
        alloc: &'chunk ASTAllocator,
    ) -> impl for<'src> FnMut(Span<'src>) -> ParseResult<'src, TableConstructor<'chunk>> {
        |input| {
            map(
                delimited(
                    pair(tag("{"), lua_whitespace0),
                    |input| parse_table_ctor(input, alloc),
                    pair(lua_whitespace0, tag("}")),
                ),
                |fields| TableConstructor { fields },
            )(input)
        }
    }
}

fn parse_table_ctor<'src, 'chunk>(
    mut input: Span<'src>,
    alloc: &'chunk ASTAllocator,
) -> ParseResult<'src, List<'chunk, Field<'chunk>>> {
    let (remain, maybe_head) = opt(Field::parser(alloc))(input)?;
    input = remain;

    let mut result = List::default();

    let mut current = if let Some(head) = maybe_head {
        result.cursor_mut().alloc_insert_advance(alloc, head)
    } else {
        let (remain, _) = opt(one_of(",;"))(input)?;
        return Ok((remain, result));
    };

    loop {
        let (remain, maybe_next) = opt(preceded(
            delimited(lua_whitespace0, one_of(",;"), lua_whitespace0),
            Field::parser(alloc),
        ))(input)?;
        input = remain;

        current = if let Some(next) = maybe_next {
            current.alloc_insert_advance(alloc, next)
        } else {
            break;
        };
    }

    let (remain, _) = opt(one_of(",;"))(input)?;

    Ok((remain, result))
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
        list::{
            List,
            ListNode,
        },
        ASTAllocator,
        Span,
    };

    #[test]
    pub fn parses_empty_table() -> anyhow::Result<()> {
        let src = "{}";

        let alloc = ASTAllocator::default();
        let result = final_parser!(Span::new(src.as_bytes()) => TableConstructor::parser(&alloc))?;

        assert_eq!(
            result,
            TableConstructor {
                fields: Default::default(),
            }
        );

        Ok(())
    }

    #[test]
    pub fn parses_empty_table_semicolon() -> anyhow::Result<()> {
        let src = "{;}";

        let alloc = ASTAllocator::default();
        let result = final_parser!(Span::new(src.as_bytes()) => TableConstructor::parser(&alloc))?;

        assert_eq!(
            result,
            TableConstructor {
                fields: Default::default(),
            }
        );
        Ok(())
    }

    #[test]
    pub fn parses_arraylike_field() -> anyhow::Result<()> {
        let src = "10";

        let alloc = ASTAllocator::default();
        let result = final_parser!(Span::new(src.as_bytes()) => Field::parser(&alloc))?;

        assert_eq!(
            result,
            Field::Arraylike {
                expression: Expression::Number(Number::Integer(10))
            }
        );

        Ok(())
    }

    #[test]
    pub fn parses_named_field() -> anyhow::Result<()> {
        let src = "a = 10";

        let alloc = ASTAllocator::default();
        let result = final_parser!(Span::new(src.as_bytes()) => Field::parser(&alloc))?;

        assert_eq!(
            result,
            Field::Named {
                name: "a".into(),
                expression: Expression::Number(Number::Integer(10))
            }
        );

        Ok(())
    }

    #[test]
    pub fn parses_index_field() -> anyhow::Result<()> {
        let src = "[11] = 10";

        let alloc = ASTAllocator::default();
        let result = final_parser!(Span::new(src.as_bytes()) => Field::parser(&alloc))?;

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
    pub fn parses_parses_table_mixed() -> anyhow::Result<()> {
        let src = "{10, [11] = 12, a = 13; 14}";

        let alloc = ASTAllocator::default();
        let result = final_parser!(Span::new(src.as_bytes()) => TableConstructor::parser(&alloc))?;

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
                        name: "a".into(),
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
