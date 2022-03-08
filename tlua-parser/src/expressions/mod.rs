use crate::{
    combinators::{
        parse_separated_list0,
        parse_separated_list_with_head,
    },
    expressions::{
        operator::Or,
        strings::ConstantString,
    },
    lexer::{
        LexedNumber,
        Token,
    },
    list::List,
    prefix_expression::{
        FnCallPrefixExpression,
        PrefixExpression,
        VarPrefixExpression,
    },
    token_subset,
    ASTAllocator,
    ParseError,
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

impl<'chunk> From<(PrefixExpression<'chunk>, &'chunk ASTAllocator)> for Expression<'chunk> {
    fn from((expr, alloc): (PrefixExpression<'chunk>, &'chunk ASTAllocator)) -> Self {
        match expr {
            PrefixExpression::Variable(var) => Self::Variable(alloc.alloc(var)),
            PrefixExpression::FnCall(call) => Self::FunctionCall(alloc.alloc(call)),
            PrefixExpression::Parenthesized(expr) => Self::Parenthesized(expr),
        }
    }
}

impl<'chunk> Expression<'chunk> {
    pub(crate) fn try_parse(
        lexer: &mut PeekableLexer,
        alloc: &'chunk ASTAllocator,
    ) -> Result<Option<Self>, ParseError> {
        // We start at the bottom of the precedence tree, and internally it'll
        // recursively move up the tree until it reaches the top.
        // After attempting to match an operator at the top of the tree, it'll move down
        // a layer, attempt to match the next operator, and if successful, move back up
        // the tree.
        Or::parse(lexer, alloc)
    }

    pub(crate) fn parse(
        lexer: &mut PeekableLexer,
        alloc: &'chunk ASTAllocator,
    ) -> Result<Self, ParseError> {
        Self::try_parse(lexer, alloc).and_then(|expr| {
            expr.ok_or_else(|| ParseError::from_here(lexer, SyntaxError::ExpectedExpression))
        })
    }

    pub(crate) fn parse_list1(
        lexer: &mut PeekableLexer,
        alloc: &'chunk ASTAllocator,
    ) -> Result<List<'chunk, Self>, ParseError> {
        let head = Self::parse(lexer, alloc)?;
        parse_separated_list_with_head(head, lexer, alloc, Self::try_parse, |token| {
            *token == Token::Comma
        })
    }

    pub(crate) fn parse_list0(
        lexer: &mut PeekableLexer,
        alloc: &'chunk ASTAllocator,
    ) -> Result<List<'chunk, Self>, ParseError> {
        parse_separated_list0(lexer, alloc, Self::try_parse, |token| {
            *token == Token::Comma
        })
    }

    fn parse_leaf(
        lexer: &mut PeekableLexer,
        alloc: &'chunk ASTAllocator,
    ) -> Result<Option<Self>, ParseError> {
        token_subset! {
            LeafToken {
                Token::Nil,
                Token::Boolean(val: bool),
                Token::HexInt(val: LexedNumber),
                Token::HexFloat(val: LexedNumber),
                Token::Int(val: LexedNumber),
                Token::Float(val: LexedNumber),
                Token::Ellipses,
                Token::KWfunction,
                Error(SyntaxError::ExpectedExpression)
            }
        }

        if let Some(string) = ConstantString::try_parse(lexer, alloc)? {
            Ok(Some(Self::String(string)))
        } else if let Some(table) = TableConstructor::try_parse(lexer, alloc)? {
            Ok(Some(Self::TableConstructor(table)))
        } else if let Some(prefix_expr) = PrefixExpression::try_parse(lexer, alloc)? {
            match prefix_expr {
                PrefixExpression::Variable(var) => Ok(Some(Self::Variable(alloc.alloc(var)))),
                PrefixExpression::FnCall(call) => Ok(Some(Self::FunctionCall(alloc.alloc(call)))),
                PrefixExpression::Parenthesized(expr) => Ok(Some(Self::Parenthesized(expr))),
            }
        } else if let Some(token) = LeafToken::next(lexer) {
            Ok(Some(match token.as_ref() {
                LeafToken::Nil => Self::Nil(Nil),
                LeafToken::Boolean(b) => Self::Bool(*b),
                LeafToken::Int(i)
                | LeafToken::Float(i)
                | LeafToken::HexFloat(i)
                | LeafToken::HexInt(i) => match i {
                    LexedNumber::Float(f) => Self::Number(Number::Float(*f)),
                    LexedNumber::Int(i) => Self::Number(Number::Integer(*i)),
                    LexedNumber::MalformedNumber => {
                        return Err(ParseError::from_here(lexer, SyntaxError::MalformedNumber));
                    }
                },
                LeafToken::Ellipses => Self::VarArgs(VarArgs),
                LeafToken::KWfunction => {
                    let body = FnBody::parse(lexer, alloc)?;
                    Self::FnDef(alloc.alloc(body))
                }
            }))
        } else {
            Ok(None)
        }
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
