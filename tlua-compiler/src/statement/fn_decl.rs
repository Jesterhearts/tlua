use scopeguard::guard_on_success;
use tlua_bytecode::{
    opcodes,
    Constant,
    OpError,
};
use tlua_parser::statement::fn_decl::FnDecl;

use crate::{
    compiler::{
        HasVaArgs,
        RegisterOps,
    },
    expressions::function_defs::emit_fn,
    CompileError,
    CompileStatement,
    Scope,
};

impl CompileStatement for FnDecl<'_> {
    fn compile(&self, scope: &mut Scope) -> Result<Option<OpError>, CompileError> {
        match self {
            FnDecl::Function { body, name } => {
                let func_id = emit_fn(
                    scope,
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

                let func = scope.push_immediate();
                func.alloc_and_set_from_fn(scope, func_id)?;
                let mut scope = guard_on_success(scope, |scope| scope.pop_immediate(func));

                if name.path.is_empty() {
                    debug_assert!(name.method.is_none());
                    return Ok(None);
                }

                let mut path = name.path.iter();

                let head = scope.read_variable(path.next().copied().expect("Path is not empty"))?;

                if path.len() == 0 && name.method.is_none() {
                    head.set_from_immediate(&mut scope, func)?;
                    return Ok(None);
                }

                let table = scope.push_immediate();
                table.set_from_local(&mut scope, head)?;
                let mut scope = guard_on_success(&mut scope, |scope| scope.pop_immediate(table));

                let index_reg = scope.push_immediate();
                let mut scope =
                    guard_on_success(&mut scope, |scope| scope.pop_immediate(index_reg));

                for _ in 0..(path.len().saturating_sub(1)) {
                    let index = path.next().expect("Still in bounds");
                    index_reg.set_from_constant(&mut scope, Constant::String(index.into()))?;
                    scope.emit(opcodes::Lookup::from((table, table, index_reg)));
                }

                match (path.next(), name.method) {
                    (Some(last), None) => {
                        index_reg.set_from_constant(&mut scope, Constant::String(last.into()))?;
                    }
                    (Some(last), Some(method)) => {
                        index_reg.set_from_constant(&mut scope, Constant::String(last.into()))?;
                        scope.emit(opcodes::Lookup::from((table, table, index_reg)));

                        index_reg.set_from_constant(&mut scope, Constant::String(method.into()))?;
                    }
                    (None, Some(method)) => {
                        index_reg.set_from_constant(&mut scope, Constant::String(method.into()))?;
                    }
                    (None, None) => unreachable!("Must have a path or a method name"),
                }

                scope.emit(opcodes::SetProperty::from((table, index_reg, func)));
            }
            FnDecl::Local { body, name } => {
                // This variable will be in scope for all child scopes :(
                // So we have to allocate a register for it here before compiling the function
                // body.
                let register = scope.new_local(*name)?;

                let fn_id = emit_fn(
                    scope,
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
                register.alloc_and_set_from_fn(scope, fn_id)?;
            }
        };

        Ok(None)
    }
}
