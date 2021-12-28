use crate::{
    ast::{
        constant_string::ConstantString,
        expressions::{
            tables::TableConstructor,
            Expression,
        },
    },
    list::List,
};

#[derive(Debug, PartialEq)]
pub enum FnArgs<'chunk> {
    Expressions(List<'chunk, Expression<'chunk>>),
    TableConstructor(TableConstructor<'chunk>),
    String(ConstantString),
}
