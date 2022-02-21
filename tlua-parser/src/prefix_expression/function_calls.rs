use nom::{
    branch::alt,
    character::complete::{char as token},
    combinator::{
        map,
        opt,
    },
    sequence::{
        delimited,
        pair,
    },
};

use crate::{
    expressions::{
        build_expression_list1,
        tables::TableConstructor,
        Expression,
    },
    list::List,
    lua_whitespace0,
    string::{
        parse_string,
        ConstantString,
    },
    ASTAllocator,
    ParseResult,
    Span,
};

#[derive(Debug, PartialEq)]
pub enum FnArgs<'chunk> {
    Expressions(List<'chunk, Expression<'chunk>>),
    TableConstructor(TableConstructor<'chunk>),
    String(ConstantString),
}

impl<'chunk> FnArgs<'chunk> {
    pub(crate) fn parser(
        alloc: &'chunk ASTAllocator,
    ) -> impl for<'src> FnMut(Span<'src>) -> ParseResult<'src, FnArgs<'chunk>> {
        |input| {
            alt((
                map(
                    delimited(
                        pair(token('('), lua_whitespace0),
                        opt(build_expression_list1(alloc)),
                        pair(lua_whitespace0, token(')')),
                    ),
                    |exprs| Self::Expressions(exprs.unwrap_or_default()),
                ),
                map(TableConstructor::parser(alloc), Self::TableConstructor),
                map(parse_string, Self::String),
            ))(input)
        }
    }
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;

    use super::FnArgs;
    use crate::{
        expressions::{
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
        Span,
    };

    #[test]
    pub fn parses_empty_args() -> anyhow::Result<()> {
        let src = "()";

        let alloc = ASTAllocator::default();
        let result = final_parser!(Span::new(src.as_bytes()) => FnArgs::parser(&alloc))?;

        assert_eq!(result, FnArgs::Expressions(Default::default()));

        Ok(())
    }

    #[test]
    pub fn parses_parenthetical_args() -> anyhow::Result<()> {
        let src = "(nil, nil, nil)";

        let alloc = ASTAllocator::default();
        let result = final_parser!(Span::new(src.as_bytes()) => FnArgs::parser(&alloc))?;

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
        let result = final_parser!(Span::new(src.as_bytes()) => FnArgs::parser(&alloc))?;

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
        let result = final_parser!(Span::new(src.as_bytes()) => FnArgs::parser(&alloc))?;

        assert_eq!(result, FnArgs::String("arg".into()));

        Ok(())
    }
}
