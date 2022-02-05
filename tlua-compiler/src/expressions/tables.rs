use tlua_bytecode::{
    opcodes,
    AnonymousRegister,
    OpError,
};
use tlua_parser::ast::{
    constant_string::ConstantString,
    expressions::tables::{
        Field,
        TableConstructor,
    },
};

use crate::{
    compiler::InitRegister,
    CompileError,
    CompileExpression,
    CompilerContext,
    NodeOutput,
};

impl CompileExpression for TableConstructor<'_> {
    fn compile(&self, compiler: &mut CompilerContext) -> Result<NodeOutput, CompileError> {
        let table = compiler.init_table();

        emit_init_sequence(compiler, table, self.fields.iter())?;

        Ok(NodeOutput::Immediate(table))
    }
}

pub(crate) fn emit_init_sequence<'a, 'f>(
    compiler: &mut CompilerContext,
    table: AnonymousRegister,
    fields: impl Iterator<Item = &'a Field<'f>>,
) -> Result<Option<OpError>, CompileError>
where
    'f: 'a,
{
    let mut arraylike = vec![];
    let mut last_field_va = false;
    for field in fields {
        match field {
            Field::Named { name, expression } => {
                let value = expression.compile(compiler)?;
                let index = compiler
                    .new_anon_reg()
                    .init_from_const(compiler, ConstantString::from(name).into());
                let value = compiler
                    .new_anon_reg()
                    .init_from_node_output(compiler, value);

                compiler.emit(opcodes::SetProperty::from((table, index, value)));
                last_field_va = false;
            }
            Field::Indexed { index, expression } => {
                let index = index.compile(compiler)?;
                let value = expression.compile(compiler)?;

                let index = compiler
                    .new_anon_reg()
                    .init_from_node_output(compiler, index);
                let value = compiler
                    .new_anon_reg()
                    .init_from_node_output(compiler, value);

                compiler.emit(opcodes::SetProperty::from((table, index, value)));

                last_field_va = false;
            }
            Field::Arraylike { expression } => {
                arraylike.push(expression.compile(compiler)?);
                last_field_va = matches!(
                    arraylike.last(),
                    Some(NodeOutput::VAStack | NodeOutput::ReturnValues)
                );
            }
        }
    }

    let (last, initializers) = if last_field_va {
        arraylike
            .split_last()
            .map(|(last, rest)| (Some(last), rest))
            .expect("Should have at least one element")
    } else {
        (None, arraylike.as_slice())
    };

    for (index, init) in initializers.iter().enumerate() {
        let value = compiler
            .new_anon_reg()
            .init_from_node_output(compiler, *init);

        let index = compiler.new_anon_reg().init_from_const(
            compiler,
            i64::try_from(index + 1)
                .map_err(|_| CompileError::TooManyTableEntries {
                    max: i64::MAX as usize,
                })?
                .into(),
        );

        compiler.emit(opcodes::SetProperty::from((table, index, value)));
    }

    if let Some(last) = last {
        match last {
            NodeOutput::ReturnValues => {
                compiler.emit(opcodes::SetAllPropertiesFromRet::from((
                    table,
                    initializers.len() + 1,
                )));
            }
            NodeOutput::VAStack => {
                compiler.emit(opcodes::SetAllPropertiesFromVa::from((
                    table,
                    initializers.len() + 1,
                )));
            }
            _ => {
                unreachable!("Only VA and return value nodes need special handling.")
            }
        }
    }

    Ok(None)
}
