use crate::{
    ast::expressions::Expression,
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
pub struct TableConstructor<'chunk> {
    /// `{ ['Exp'] ='Exp' }`
    /// `{ 'Name' ='Exp' }`
    pub indexed_fields: List<'chunk, (Expression<'chunk>, Expression<'chunk>)>,
    /// `{ 'Exp' }`
    ///
    /// `{ 'Exp1', 'Exp2' } ` behaves like `['Exp1', 'Exp2']` with 1-based
    /// indexing.
    pub arraylike_fields: List<'chunk, Expression<'chunk>>,
}
