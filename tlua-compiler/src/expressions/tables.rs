use tlua_parser::ast::expressions::tables::{
    Field,
    TableConstructor,
};

use crate::{
    CompileError,
    CompileExpression,
    CompilerContext,
    NodeOutput,
};

impl CompileExpression for TableConstructor<'_> {
    fn compile(&self, compiler: &mut CompilerContext) -> Result<NodeOutput, CompileError> {
        let table = compiler.init_table();

        let mut arraylike = vec![];

        let mut last_field_va = false;

        for field in self.fields.iter() {
            match field {
                Field::Named { name, expression } => {
                    compiler.assign_to_table(table, name, expression)?;
                    last_field_va = false;
                }
                Field::Indexed { index, expression } => {
                    compiler.assign_to_table(table, index, expression)?;
                    last_field_va = false;
                }
                Field::Arraylike { expression } => {
                    arraylike.push(expression.compile(compiler)?);
                    last_field_va = matches!(arraylike.last(), Some(NodeOutput::VAStack));
                }
            }
        }

        let (last, rest) = if last_field_va {
            arraylike
                .split_last()
                .map(|(last, rest)| (Some(last), rest))
                .expect("Should have at least one element")
        } else {
            (None, arraylike.as_slice())
        };

        // Arraylike fields must be stored after indexed fields to respect lua's order
        // of initialization.
        for (index, init) in rest.iter().enumerate() {
            compiler.assign_to_array(table, index, *init)?;
        }

        if let Some(last) = last {
            debug_assert!(matches!(last, NodeOutput::VAStack));
            compiler.copy_va_to_array(table, rest.len());
        }

        Ok(NodeOutput::Register(table))
    }
}
