use enum_dispatch::enum_dispatch;

use crate::ast::{
    block::Block,
    identifiers::Ident,
    prefix_expression::FnCallPrefixExpression,
};

pub mod assignment;
pub mod fn_decl;
pub mod for_loop;
pub mod foreach_loop;
pub mod if_statement;
pub mod repeat_loop;
pub mod variables;
pub mod while_loop;

use self::{
    assignment::Assignment,
    fn_decl::FnDecl,
    for_loop::ForLoop,
    foreach_loop::ForEachLoop,
    if_statement::If,
    repeat_loop::RepeatLoop,
    variables::LocalVarList,
    while_loop::WhileLoop,
};

#[derive(Debug, PartialEq)]
pub(crate) struct Empty;

#[derive(Debug, PartialEq)]
pub(crate) struct Break;

#[derive(Debug, PartialEq)]
pub(crate) struct Label(pub(crate) Ident);

#[derive(Debug, PartialEq)]
pub(crate) struct Goto(pub(crate) Ident);

#[enum_dispatch]
#[derive(Debug, PartialEq)]
pub(crate) enum Statement<'chunk> {
    Empty,
    Assignment(&'chunk Assignment<'chunk>),
    Call(&'chunk FnCallPrefixExpression<'chunk>),
    // TODO(lang-5.4): Scoping & matching rules.
    Label,
    Break,
    // TODO(lang-5.4): Scoping & matching rules.
    Goto,
    Do(&'chunk Block<'chunk>),
    While(&'chunk WhileLoop<'chunk>),
    Repeat(&'chunk RepeatLoop<'chunk>),
    If(&'chunk If<'chunk>),
    For(&'chunk ForLoop<'chunk>),
    ForEach(&'chunk ForEachLoop<'chunk>),
    FnDecl(&'chunk FnDecl<'chunk>),
    LocalVarList(&'chunk LocalVarList<'chunk>),
}

#[cfg(test)]
mod tests {
    use pretty_assertions::Comparison;

    use super::Statement;

    #[test]
    fn sizeof_statement() {
        let left = std::mem::size_of::<Statement>();
        let right = std::mem::size_of::<usize>() * 4;
        if left > right {
            panic!(
                "assertion failed: `(left <= right)`\
                        \n\
                        \n{}:\
                        \n\
                        \n",
                Comparison::new(&left, &right)
            );
        }
    }
}
