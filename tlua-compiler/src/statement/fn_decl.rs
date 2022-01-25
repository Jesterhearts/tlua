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
    CompileError,
    CompileStatement,
    CompilerContext,
    TypeIds,
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
                    body.params.named_params.iter().copied(),
                    body.body.statements.iter(),
                    body.body.ret.as_ref(),
                )?;

                if name.path.is_empty() {
                    let _func = compiler.new_anon_reg().init_alloc_fn(compiler, func_id);
                    return Ok(None);
                }

                let mut path = name.path.iter();
                let reg =
                    compiler.read_variable(path.next().copied().expect("Path is not empty"))?;

                if path.len() == 1 {
                    compiler.emit(opcodes::Alloc::from((reg, TypeIds::FUNCTION, func_id)));
                    return Ok(None);
                }

                todo!()
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
                    body.params.named_params.iter().copied(),
                    body.body.statements.iter(),
                    body.body.ret.as_ref(),
                )?;

                // Because this is a local function declaration, we know we're the first write
                // to it in scope. We had to have the register already allocated though so it
                // could be in scope during compilation of child scopes.
                register.init_alloc_fn(compiler, fn_id);

                Ok(None)
            }
        }
    }
}
