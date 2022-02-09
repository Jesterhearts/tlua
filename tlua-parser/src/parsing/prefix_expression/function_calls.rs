use nom::{
    branch::alt,
    bytes::complete::tag,
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
    ast::{
        expressions::tables::TableConstructor,
        prefix_expression::function_calls::FnArgs,
    },
    parsing::{
        expressions::expression_list1,
        lua_whitespace0,
        string::parse_string,
        ASTAllocator,
        Parse,
        ParseResult,
        Span,
    },
};

impl<'chunk> Parse<'chunk> for FnArgs<'chunk> {
    fn parse<'src>(input: Span<'src>, alloc: &'chunk ASTAllocator) -> ParseResult<'src, Self> {
        alt((
            map(
                delimited(
                    pair(tag("("), lua_whitespace0),
                    opt(|input| expression_list1(input, alloc)),
                    pair(lua_whitespace0, tag(")")),
                ),
                |exprs| Self::Expressions(exprs.unwrap_or_default()),
            ),
            map(
                |input| TableConstructor::parse(input, alloc),
                Self::TableConstructor,
            ),
            map(parse_string, Self::String),
        ))(input)
    }
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;

    use super::FnArgs;
    use crate::{
        ast::expressions::{
            tables::TableConstructor,
            Expression,
            Nil,
        },
        list::{
            List,
            ListNode,
        },
        parsing::{
            ASTAllocator,
            Parse,
            Span,
        },
    };

    #[test]
    pub fn parses_empty_args() -> anyhow::Result<()> {
        let src = "()";

        let alloc = ASTAllocator::default();
        let (remain, result) = FnArgs::parse(Span::new(src.as_bytes()), &alloc)?;

        assert_eq!(std::str::from_utf8(*remain)?, "");
        assert_eq!(result, FnArgs::Expressions(Default::default()));

        Ok(())
    }

    #[test]
    pub fn parses_parenthetical_args() -> anyhow::Result<()> {
        let src = "(nil, nil, nil)";

        let alloc = ASTAllocator::default();
        let (remain, result) = FnArgs::parse(Span::new(src.as_bytes()), &alloc)?;

        assert_eq!(std::str::from_utf8(*remain)?, "");
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
        let (remain, result) = FnArgs::parse(Span::new(src.as_bytes()), &alloc)?;

        assert_eq!(std::str::from_utf8(*remain)?, "");
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
        let (remain, result) = FnArgs::parse(Span::new(src.as_bytes()), &alloc)?;

        assert_eq!(std::str::from_utf8(*remain)?, "");
        assert_eq!(result, FnArgs::String("arg".into()));

        Ok(())
    }
}
