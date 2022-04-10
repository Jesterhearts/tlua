use std::{
    marker::PhantomData,
    ops::Range,
};

use derive_more::From;
use tlua_bytecode::{
    opcodes,
    ImmediateRegister,
};
use tlua_parser::{
    block::Block,
    identifiers::Ident,
    StringTable,
};

use crate::{
    block::emit_block,
    Chunk,
    CompileError,
};

mod register;
mod scope;
pub(super) mod unasm;

pub(crate) use register::RegisterOps;
pub(crate) use scope::Scope;

use self::{
    scope::*,
    unasm::*,
};

#[derive(Debug, Clone, Copy)]
pub(crate) enum HasVaArgs {
    None,
    Some,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) enum LabelId {
    Named(Ident),
    If { id: usize },
    Loop { id: usize },
}

#[derive(Debug)]
pub(crate) struct Compiler {
    root: RootScope,
}

impl Compiler {
    pub(crate) fn new(strings: StringTable) -> Self {
        Self {
            root: RootScope::new(strings),
        }
    }

    pub(crate) fn compile_ast(mut self, ast: Block) -> Result<Chunk, CompileError> {
        let main = {
            let mut main = self.root.start_main();
            {
                let mut block = main.start();
                let mut scope = block.enter();

                emit_block(&mut scope, &ast)?;
            }

            main.complete_main()
        };

        Ok(self.root.into_chunk(main))
    }
}

#[derive(Debug)]
pub(crate) enum JumpTemplate<Op> {
    Unconditional {
        location: usize,
    },
    Conditional {
        location: usize,
        reg: ImmediateRegister,
        op: PhantomData<Op>,
    },
}

impl<Op: From<(ImmediateRegister, usize)> + Into<UnasmOp>> JumpTemplate<Op> {
    pub(crate) fn unconditional_at(location: usize) -> Self {
        Self::Unconditional { location }
    }

    pub(crate) fn conditional_at(location: usize, reg: ImmediateRegister) -> Self {
        Self::Conditional {
            location,
            reg,
            op: Default::default(),
        }
    }

    pub(crate) fn resolve_to(self, target: usize, scope: &mut Scope) {
        match self {
            JumpTemplate::Unconditional { location } => {
                scope.overwrite(location, opcodes::Jump::from(target))
            }
            JumpTemplate::Conditional {
                location,
                reg,
                op: _,
            } => scope.overwrite(location, Op::from((reg, target))),
        }
    }
}

#[derive(Debug, Clone, From)]
pub(crate) struct RegisterRange {
    range: Range<usize>,
}

impl RegisterRange {
    pub(crate) fn iter(&self) -> impl ExactSizeIterator<Item = ImmediateRegister> {
        self.range.clone().map(ImmediateRegister::from)
    }
}
