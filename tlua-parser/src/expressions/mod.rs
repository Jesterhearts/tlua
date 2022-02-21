use nom::{
    branch::alt,
    bytes::complete::tag,
    character::complete::char as token,
    combinator::{
        cut,
        map,
    },
    sequence::{
        delimited,
        pair,
        preceded,
    },
};

use crate::{
    build_separated_list0,
    build_separated_list1,
    identifiers::keyword,
    list::List,
    lua_whitespace0,
    prefix_expression::{
        FnCallPrefixExpression,
        PrefixExpression,
        VarPrefixExpression,
    },
    string::{
        parse_string,
        ConstantString,
    },
    ASTAllocator,
    ParseResult,
    Span,
};

pub mod constants;
pub mod function_defs;
pub mod number;
pub mod operator;
pub mod tables;

use self::{
    constants::{
        parse_bool,
        parse_nil,
    },
    function_defs::FnBody,
    number::{
        parse_number,
        Number,
    },
    operator::{
        parse_or_expr,
        BinaryOperator,
        UnaryOperator,
    },
    tables::TableConstructor,
};

#[derive(Debug, PartialEq, Clone, Copy)]
pub struct Nil;

#[derive(Debug, PartialEq)]
pub struct VarArgs;

#[derive(Debug, PartialEq)]
pub enum Expression<'chunk> {
    Parenthesized(&'chunk Expression<'chunk>),
    Variable(&'chunk VarPrefixExpression<'chunk>),
    FunctionCall(&'chunk FnCallPrefixExpression<'chunk>),
    Nil(Nil),
    Bool(bool),
    Number(Number),
    String(ConstantString),
    FnDef(&'chunk FnBody<'chunk>),
    TableConstructor(TableConstructor<'chunk>),
    VarArgs(VarArgs),
    BinaryOp(BinaryOperator<'chunk>),
    UnaryOp(UnaryOperator<'chunk>),
}

impl<'chunk> Expression<'chunk> {
    pub(crate) fn parser(
        alloc: &'chunk ASTAllocator,
    ) -> impl for<'src> FnMut(Span<'src>) -> ParseResult<'src, Expression<'chunk>> {
        // This is a hidden alt statement essentially.
        // We start at the bottom of the precedence tree, and internally it'll
        // recursively move up the tree until it reaches the top.
        // After attempting to match an operator at the top of the tree, it'll move down
        // a layer, attempt to match the next operator, and if successful, move back up
        // the tree.
        |input| parse_or_expr(input, alloc)
    }
}

pub fn build_expression_list0<'chunk>(
    alloc: &'chunk ASTAllocator,
) -> impl for<'src> FnMut(Span<'src>) -> ParseResult<'src, List<'chunk, Expression<'chunk>>> {
    |input| {
        build_separated_list0(
            alloc,
            Expression::parser(alloc),
            delimited(lua_whitespace0, token(','), lua_whitespace0),
        )(input)
    }
}

pub fn build_expression_list1<'chunk>(
    alloc: &'chunk ASTAllocator,
) -> impl for<'src> FnMut(Span<'src>) -> ParseResult<'src, List<'chunk, Expression<'chunk>>> {
    |input| {
        build_separated_list1(
            alloc,
            Expression::parser(alloc),
            delimited(lua_whitespace0, token(','), lua_whitespace0),
        )(input)
    }
}

fn parse_non_op_expr<'src, 'chunk>(
    input: Span<'src>,
    alloc: &'chunk ASTAllocator,
) -> ParseResult<'src, Expression<'chunk>> {
    alt((
        map(parse_nil, Expression::Nil),
        map(parse_bool, Expression::Bool),
        map(parse_number, Expression::Number),
        map(parse_string, Expression::String),
        map(tag("..."), |_| Expression::VarArgs(VarArgs)),
        map(
            preceded(
                pair(keyword("function"), lua_whitespace0),
                cut(FnBody::parser(alloc)),
            ),
            |body| Expression::FnDef(alloc.alloc(body)),
        ),
        map(PrefixExpression::parser(alloc), |expr| match expr {
            PrefixExpression::Variable(var) => Expression::Variable(alloc.alloc(var)),
            PrefixExpression::FnCall(call) => Expression::FunctionCall(alloc.alloc(call)),
            PrefixExpression::Parenthesized(expr) => Expression::Parenthesized(expr),
        }),
        map(
            TableConstructor::parser(alloc),
            Expression::TableConstructor,
        ),
    ))(input)
}

#[cfg(test)]
mod tests {
    use pretty_assertions::Comparison;

    use super::Expression;
    use crate::{
        expressions::{
            tables::TableConstructor,
            VarArgs,
        },
        final_parser,
        ASTAllocator,
        Span,
    };

    #[test]
    pub fn parses_varargs() -> anyhow::Result<()> {
        let src = "...";

        let alloc = ASTAllocator::default();
        let result = final_parser!(Span::new(src.as_bytes()) => Expression::parser(&alloc))?;
        assert_eq!(result, Expression::VarArgs(VarArgs));

        Ok(())
    }

    #[test]
    pub fn parses_table() -> anyhow::Result<()> {
        let src = "{}";

        let alloc = ASTAllocator::default();
        let result = final_parser!(Span::new(src.as_bytes()) => Expression::parser(&alloc))?;
        assert_eq!(
            result,
            Expression::TableConstructor(TableConstructor {
                fields: Default::default(),
            })
        );

        Ok(())
    }

    #[test]
    fn sizeof_expr() {
        let left = std::mem::size_of::<Expression>();
        let right = std::mem::size_of::<usize>() * 4;
        if left > right {
            panic!(
                "assertion failed: `(left <= right)`\
                        \n\
                        \n{}:\
                        \n\
                        \n",
                Comparison::new(&left, &right)
            );
        }
    }
}
