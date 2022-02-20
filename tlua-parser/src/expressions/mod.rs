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
        preceded,
    },
};

use crate::{
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
    Parse,
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

impl<'chunk> Parse<'chunk> for Expression<'chunk> {
    fn parse<'src>(input: Span<'src>, alloc: &'chunk ASTAllocator) -> ParseResult<'src, Self> {
        // This is a hidden alt statement essentially.
        // We start at the bottom of the precedence tree, and internally it'll
        // recursively move up the tree until it reaches the top.
        // After attempting to match an operator at the top of the tree, it'll move down
        // a layer, attempt to match the next operator, and if successful, move back up
        // the tree.
        parse_or_expr(input, alloc)
    }
}

pub fn expression_list1<'src, 'chunk>(
    mut input: Span<'src>,
    alloc: &'chunk ASTAllocator,
) -> ParseResult<'src, List<'chunk, Expression<'chunk>>> {
    let (remain, expr) = Expression::parse(input, alloc)?;
    input = remain;

    let mut list = List::default();
    let mut current = list.cursor_mut().alloc_insert_advance(alloc, expr);

    loop {
        let (remain, maybe_next) = opt(preceded(
            delimited(lua_whitespace0, tag(","), lua_whitespace0),
            |input| Expression::parse(input, alloc),
        ))(input)?;
        input = remain;

        current = if let Some(next) = maybe_next {
            current.alloc_insert_advance(alloc, next)
        } else {
            return Ok((input, list));
        };
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
            preceded(pair(tag("function"), lua_whitespace0), |input| {
                FnBody::parse(input, alloc)
            }),
            |body| Expression::FnDef(alloc.alloc(body)),
        ),
        map(
            |input| PrefixExpression::parse(input, alloc),
            |expr| match expr {
                PrefixExpression::Variable(var) => Expression::Variable(alloc.alloc(var)),
                PrefixExpression::FnCall(call) => Expression::FunctionCall(alloc.alloc(call)),
                PrefixExpression::Parenthesized(expr) => Expression::Parenthesized(expr),
            },
        ),
        map(
            |input| TableConstructor::parse(input, alloc),
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
        Parse,
        Span,
    };

    #[test]
    pub fn parses_varargs() -> anyhow::Result<()> {
        let src = "...";

        let alloc = ASTAllocator::default();
        let result =
            final_parser!(Span::new(src.as_bytes()) => |input| Expression::parse(input, &alloc))?;
        assert_eq!(result, Expression::VarArgs(VarArgs));

        Ok(())
    }

    #[test]
    pub fn parses_table() -> anyhow::Result<()> {
        let src = "{}";

        let alloc = ASTAllocator::default();
        let result =
            final_parser!(Span::new(src.as_bytes()) => |input| Expression::parse(input, &alloc))?;
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
