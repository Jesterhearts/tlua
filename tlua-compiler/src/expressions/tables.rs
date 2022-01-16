use tlua_bytecode::OpError;
use tlua_parser::ast::{
    constant_string::ConstantString,
    expressions::tables::{
        Field,
        TableConstructor,
    },
};

use crate::{
    compiler::unasm::UnasmRegister,
    CompileError,
    CompileExpression,
    CompilerContext,
    NodeOutput,
};

impl CompileExpression for TableConstructor<'_> {
    fn compile(&self, compiler: &mut CompilerContext) -> Result<NodeOutput, CompileError> {
        let table = compiler.init_table();

        emit_init_sequence(compiler, table, self.fields.iter())?;

        Ok(NodeOutput::Register(table))
    }
}

pub(crate) fn emit_init_sequence<'a, 'f>(
    compiler: &mut CompilerContext,
    table: UnasmRegister,
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
                if let err @ Some(_) =
                    compiler.assign_to_table(table, ConstantString::from(name), expression)?
                {
                    return Ok(err);
                }
                last_field_va = false;
            }
            Field::Indexed { index, expression } => {
                if let err @ Some(_) = compiler.assign_to_table(table, index, expression)? {
                    return Ok(err);
                }
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

    let (last, rest) = if last_field_va {
        arraylike
            .split_last()
            .map(|(last, rest)| (Some(last), rest))
            .expect("Should have at least one element")
    } else {
        (None, arraylike.as_slice())
    };

    for (index, init) in rest.iter().enumerate() {
        compiler.assign_to_array(table, index, *init)?;
    }

    if let Some(last) = last {
        match last {
            NodeOutput::ReturnValues => compiler.copy_ret_to_array(table, rest.len()),
            NodeOutput::VAStack => compiler.copy_va_to_array(table, rest.len()),
            NodeOutput::Constant(_) | NodeOutput::Register(_) | NodeOutput::Err(_) => {
                unreachable!("Only VA and return value nodes need special handling.")
            }
        }
    }

    Ok(None)
}
