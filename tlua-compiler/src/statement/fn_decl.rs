use tlua_bytecode::{
    opcodes,
    OpError,
};
use tlua_parser::ast::statement::fn_decl::FnDecl;

use crate::{
    compiler::{
        HasVaArgs,
        InitRegister,
    },
    constant::Constant,
    CompileError,
    CompileStatement,
    CompilerContext,
};

impl CompileStatement for FnDecl<'_> {
    fn compile(&self, compiler: &mut CompilerContext) -> Result<Option<OpError>, CompileError> {
        match self {
            FnDecl::Function { body, name } => {
                let func_id = compiler.emit_fn(
                    if body.params.varargs {
                        HasVaArgs::Some
                    } else {
                        HasVaArgs::None
                    },
                    name.method.is_some(),
                    body.params.named_params.iter().copied(),
                    body.body.statements.iter(),
                    body.body.ret.as_ref(),
                )?;

                let func = compiler.new_anon_reg().init_alloc_fn(compiler, func_id);
                if name.path.is_empty() {
                    debug_assert!(name.method.is_none());
                    return Ok(None);
                }

                let mut path = name.path.iter();

                let head =
                    compiler.read_variable(path.next().copied().expect("Path is not empty"))?;

                if path.len() == 0 && name.method.is_none() {
                    head.init_from_anon_reg(compiler, func);
                    return Ok(None);
                }

                let table = compiler.new_anon_reg().init_from_mapped_reg(compiler, head);
                let index_reg = compiler.new_anon_reg().no_init_needed();

                for _ in 0..(path.len().saturating_sub(1)) {
                    let index = path.next().expect("Still in bounds");
                    index_reg.init_from_const(compiler, Constant::String(index.into()));
                    compiler.emit(opcodes::Lookup::from((table, table, index_reg)));
                }

                match (path.next(), name.method) {
                    (Some(last), None) => {
                        index_reg.init_from_const(compiler, Constant::String(last.into()));
                    }
                    (Some(last), Some(method)) => {
                        index_reg.init_from_const(compiler, Constant::String(last.into()));
                        compiler.emit(opcodes::Lookup::from((table, table, index_reg)));

                        index_reg.init_from_const(compiler, Constant::String(method.into()));
                    }
                    (None, Some(method)) => {
                        index_reg.init_from_const(compiler, Constant::String(method.into()));
                    }
                    (None, None) => unreachable!("Must have a path or a method name"),
                }

                compiler.emit(opcodes::SetProperty::from((table, index_reg, func)));
            }
            FnDecl::Local { body, name } => {
                // This variable will be in scope for all child scopes :(
                // So we have to allocate a register for it here before compiling the function
                // body.
                let register = compiler.new_local(*name)?;

                let fn_id = compiler.emit_fn(
                    if body.params.varargs {
                        HasVaArgs::Some
                    } else {
                        HasVaArgs::None
                    },
                    false,
                    body.params.named_params.iter().copied(),
                    body.body.statements.iter(),
                    body.body.ret.as_ref(),
                )?;

                // Because this is a local function declaration, we know we're the first write
                // to it in scope. We had to have the register already allocated though so it
                // could be in scope during compilation of child scopes.
                register.init_alloc_fn(compiler, fn_id);
            }
        };

        Ok(None)
    }
}
