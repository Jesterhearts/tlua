use crate::{
    ast::{
        expressions::Expression,
        prefix_expression::VarPrefixExpression,
    },
    list::List,
};

#[derive(Debug, PartialEq)]
pub(crate) struct Assignment<'chunk> {
    pub(crate) varlist: List<'chunk, VarPrefixExpression<'chunk>>,
    pub(crate) expressions: List<'chunk, Expression<'chunk>>,
}
