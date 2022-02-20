use crate::{
    expressions::Expression,
    list::List,
    prefix_expression::VarPrefixExpression,
};

#[derive(Debug, PartialEq)]
pub struct Assignment<'chunk> {
    pub varlist: List<'chunk, VarPrefixExpression<'chunk>>,
    pub expressions: List<'chunk, Expression<'chunk>>,
}
