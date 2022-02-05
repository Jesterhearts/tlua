use tlua_bytecode::OpError;
use tlua_parser::ast::statement::variables::LocalVarList;

use crate::{
    compiler::InitRegister,
    statement::assignment,
    CompileError,
    CompileStatement,
    CompilerContext,
};

impl CompileStatement for LocalVarList<'_> {
    fn compile(&self, compiler: &mut CompilerContext) -> Result<Option<OpError>, CompileError> {
        assignment::emit_assignments(
            compiler,
            |compiler, var| match var.attribute {
                None => compiler.new_local(var.name),
                Some(_) => todo!(),
            },
            |compiler, reg, src| {
                let src = compiler.output_to_reg_reuse_anon(src);
                reg.init_from_anon_reg(compiler, src);
            },
            self.vars.iter(),
            self.initializers.iter(),
        )
    }
}
