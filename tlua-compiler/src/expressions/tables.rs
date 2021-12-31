use tlua_parser::ast::expressions::tables::TableConstructor;

use crate::{
    CompileError,
    CompileExpression,
    CompilerContext,
    NodeOutput,
};

impl CompileExpression for TableConstructor<'_> {
    fn compile(&self, compiler: &mut CompilerContext) -> Result<NodeOutput, CompileError> {
        let reg = compiler.init_table();

        for (index, init) in self.indexed_fields.iter() {
            compiler.assign_to_table(reg.into(), index, init)?;
        }

        for (index, init) in self.arraylike_fields.iter().enumerate() {
            compiler.assign_to_array(reg.into(), index, init)?;
        }

        Ok(NodeOutput::Register(reg.into()))
    }
}
