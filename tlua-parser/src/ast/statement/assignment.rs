use crate::{
    ast::{
        expressions::Expression,
        prefix_expression::VarPrefixExpression,
    },
    list::List,
};

#[derive(Debug, PartialEq)]
pub struct Assignment<'chunk> {
    pub varlist: List<'chunk, VarPrefixExpression<'chunk>>,
    pub expressions: List<'chunk, Expression<'chunk>>,
}
