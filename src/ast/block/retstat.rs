use crate::{
    ast::expressions::Expression,
    list::List,
};

#[derive(Debug, PartialEq)]
pub(crate) struct RetStatement<'chunk> {
    pub(crate) expressions: List<'chunk, Expression<'chunk>>,
}
