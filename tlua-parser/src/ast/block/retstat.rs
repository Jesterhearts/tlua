use crate::{
    ast::expressions::Expression,
    list::List,
};

#[derive(Debug, PartialEq)]
pub struct RetStatement<'chunk> {
    pub expressions: List<'chunk, Expression<'chunk>>,
}
