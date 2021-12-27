use crate::{
    ast::statement::Statement,
    list::List,
};

pub mod retstat;
use self::retstat::RetStatement;

#[derive(Debug, Default, PartialEq)]
pub struct Block<'chunk> {
    pub(crate) statements: List<'chunk, Statement<'chunk>>,
    pub(crate) ret: Option<RetStatement<'chunk>>,
}
