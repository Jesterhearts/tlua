use scopeguard::guard_on_success;
use tlua_bytecode::OpError;
use tlua_parser::ast::statement::variables::LocalVarList;

use crate::{
    compiler::InitRegister,
    statement::assignment,
    CompileError,
    CompileStatement,
    Scope,
};

impl CompileStatement for LocalVarList<'_> {
    fn compile(&self, scope: &mut Scope) -> Result<Option<OpError>, CompileError> {
        assignment::emit_assignments(
            scope,
            |scope, var| match var.attribute {
                None => scope.new_local(var.name),
                Some(_) => todo!(),
            },
            |scope, reg, src| {
                let src = src.into_register(scope);
                let mut scope = guard_on_success(scope, |scope| scope.pop_immediate(src));
                reg.init_from_immediate(&mut scope, src);
            },
            self.vars.iter(),
            self.initializers.iter(),
        )
    }
}
