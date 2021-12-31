use crate::{
    ast::{
        expressions::Expression,
        identifiers::Ident,
    },
    list::List,
};

/// Field values for a field list ordered in ascending order of precedence.
///
/// If you have an expression like:
/// ```lua
/// {10, 11, [1] = 13}
/// -- alternatively
/// {[1] = 13, 10, 11}
/// ```
/// Your final table will always contain `{10, 11}` as of Lua 5.4
#[derive(Debug, PartialEq)]
pub enum Field<'chunk> {
    /// `{ 'Name' ='Exp' }`
    Named {
        name: Ident,
        expression: Expression<'chunk>,
    },
    /// `{ ['Exp'] ='Exp' }`
    Indexed {
        index: Expression<'chunk>,
        expression: Expression<'chunk>,
    },
    /// `{ 'Exp' }`
    ///
    /// `{ 'Exp1', 'Exp2' } ` behaves like `['Exp1', 'Exp2']` with 1-based
    /// indexing.
    Arraylike { expression: Expression<'chunk> },
}

#[derive(Debug, PartialEq)]
pub struct TableConstructor<'chunk> {
    pub fields: List<'chunk, Field<'chunk>>,
}
