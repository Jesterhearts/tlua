use crate::ast::{
    expressions::function_defs::{
        FnBody,
        FnName,
    },
    identifiers::Ident,
};

#[derive(Debug, PartialEq)]
pub enum FnDecl<'chunk> {
    Function {
        name: FnName<'chunk>,
        body: FnBody<'chunk>,
    },
    Local {
        name: Ident,
        body: FnBody<'chunk>,
    },
}
