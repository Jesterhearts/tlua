use crate::{
    expressions::{
        operator::Or,
        strings::ConstantString,
    },
    lexer::{
        LexedNumber,
        Token,
    },
    list::List,
    parse_separated_list1,
    prefix_expression::{
        FnCallPrefixExpression,
        PrefixExpression,
        VarPrefixExpression,
    },
    ASTAllocator,
    ParseError,
    ParseErrorExt,
    PeekableLexer,
    SyntaxError,
};

pub mod function_defs;
pub mod number;
pub mod operator;
pub mod strings;
pub mod tables;

use self::{
    function_defs::FnBody,
    number::Number,
    operator::{
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
    pub(crate) fn parse(
        lexer: &mut PeekableLexer,
        alloc: &'chunk ASTAllocator,
    ) -> Result<Self, ParseError> {
        // We start at the bottom of the precedence tree, and internally it'll
        // recursively move up the tree until it reaches the top.
        // After attempting to match an operator at the top of the tree, it'll move down
        // a layer, attempt to match the next operator, and if successful, move back up
        // the tree.
        Or::parse(lexer, alloc)
    }

    pub(crate) fn parse_list1(
        lexer: &mut PeekableLexer,
        alloc: &'chunk ASTAllocator,
    ) -> Result<List<'chunk, Self>, ParseError> {
        parse_separated_list1(lexer, alloc, Self::parse, |token| *token == Token::Comma)
    }

    pub(crate) fn parse_list0(
        lexer: &mut PeekableLexer,
        alloc: &'chunk ASTAllocator,
    ) -> Result<List<'chunk, Self>, ParseError> {
        Self::parse_list1(lexer, alloc)
            .recover()
            .map(Option::unwrap_or_default)
    }

    fn parse_leaf(
        lexer: &mut PeekableLexer,
        alloc: &'chunk ASTAllocator,
    ) -> Result<Self, ParseError> {
        ConstantString::parse(lexer, alloc)
            .map(Self::String)
            .recover_with(|| TableConstructor::parse(lexer, alloc).map(Self::TableConstructor))
            .recover_with(|| {
                PrefixExpression::parse(lexer, alloc).map(|prefix_expr| match prefix_expr {
                    PrefixExpression::Variable(var) => Self::Variable(alloc.alloc(var)),
                    PrefixExpression::FnCall(call) => Self::FunctionCall(alloc.alloc(call)),
                    PrefixExpression::Parenthesized(expr) => Self::Parenthesized(expr),
                })
            })
            .recover_with(|| {
                let token = lexer
                    .next_if(|token| {
                        matches!(
                            token.as_ref(),
                            Token::Nil
                                | Token::Boolean(_)
                                | Token::HexInt(_)
                                | Token::HexFloat(_)
                                | Token::Int(_)
                                | Token::Float(_)
                                | Token::Ellipses
                                | Token::KWfunction
                        )
                    })
                    .ok_or_else(|| {
                        ParseError::recoverable_from_here(lexer, SyntaxError::ExpectedExpression)
                    })?;

                Ok(match token.into() {
                    Token::Nil => Self::Nil(Nil),
                    Token::Boolean(b) => Self::Bool(b),
                    Token::Int(i) | Token::Float(i) | Token::HexFloat(i) | Token::HexInt(i) => {
                        match i {
                            LexedNumber::Float(f) => Self::Number(Number::Float(f)),
                            LexedNumber::Int(i) => Self::Number(Number::Integer(i)),
                            LexedNumber::MalformedNumber => {
                                return Err(ParseError::unrecoverable_from_here(
                                    lexer,
                                    SyntaxError::MalformedNumber,
                                ));
                            }
                        }
                    }
                    Token::Ellipses => Self::VarArgs(VarArgs),
                    Token::KWfunction => {
                        let body = FnBody::parse(lexer, alloc).ok_or_else(|| {
                            ParseError::unrecoverable_from_here(
                                lexer,
                                SyntaxError::ExpectedFunctionDef,
                            )
                        })?;
                        Self::FnDef(alloc.alloc(body))
                    }
                    _ => unreachable!(),
                })
            })
    }
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
        StringTable,
    };

    #[test]
    pub fn parses_varargs() -> anyhow::Result<()> {
        let src = "...";

        let alloc = ASTAllocator::default();
        let mut strings = StringTable::default();
        let result = final_parser!((src.as_bytes(), &alloc, &mut strings) => Expression::parse)?;
        assert_eq!(result, Expression::VarArgs(VarArgs));

        Ok(())
    }

    #[test]
    pub fn parses_table() -> anyhow::Result<()> {
        let src = "{}";

        let alloc = ASTAllocator::default();
        let mut strings = StringTable::default();
        let result = final_parser!((src.as_bytes(), &alloc, &mut strings) => Expression::parse)?;
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
