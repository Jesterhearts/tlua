use crate::{
    ast::statement::Statement,
    list::List,
};

pub mod retstat;
use self::retstat::RetStatement;

#[derive(Debug, Default, PartialEq)]
pub struct Block<'chunk> {
    pub statements: List<'chunk, Statement<'chunk>>,
    pub ret: Option<RetStatement<'chunk>>,
}
