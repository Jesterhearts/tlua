use crate::{
    ast::{
        expressions::Expression,
        identifiers::Ident,
    },
    list::List,
};

/// Field values for a field list ordered by precedence.
#[derive(Debug, PartialEq)]
pub enum Field<'chunk> {
    /// If you have an expression like:
    /// ```lua
    /// {10, 11, [1] = 13}
    /// -- alternatively
    /// {[1] = 13, 10, 11}
    /// ```
    /// Your final table will always contain {10, 11} as of lua 5.4
    Arraylike { expression: Expression<'chunk> },
    Named {
        name: Ident,
        expression: Expression<'chunk>,
    },
    Indexed {
        index: Expression<'chunk>,
        expression: Expression<'chunk>,
    },
}

#[derive(Debug, PartialEq)]
pub struct TableConstructor<'chunk> {
    pub fields: List<'chunk, Field<'chunk>>,
}
