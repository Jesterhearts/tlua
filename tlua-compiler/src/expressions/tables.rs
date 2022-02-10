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
    NodeOutput,
    Scope,
};

impl CompileExpression for TableConstructor<'_> {
    fn compile(&self, scope: &mut Scope) -> Result<NodeOutput, CompileError> {
        let table = scope.new_anon_reg().init_alloc_table(scope);

        emit_init_sequence(scope, table, self.fields.iter())?;

        Ok(NodeOutput::Immediate(table))
    }
}

pub(crate) fn emit_init_sequence<'a, 'f>(
    scope: &mut Scope,
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
                let index = scope
                    .new_anon_reg()
                    .init_from_const(scope, ConstantString::from(name).into());

                let value = expression.compile(scope)?;

                let value = scope.new_anon_reg().init_from_node_output(scope, value);

                scope.emit(opcodes::SetProperty::from((table, index, value)));
                last_field_va = false;
            }
            Field::Indexed { index, expression } => {
                let index = index.compile(scope)?;
                let index = scope.new_anon_reg().init_from_node_output(scope, index);

                let value = expression.compile(scope)?;
                let value = scope.new_anon_reg().init_from_node_output(scope, value);

                scope.emit(opcodes::SetProperty::from((table, index, value)));

                last_field_va = false;
            }
            Field::Arraylike { expression } => {
                arraylike.push(expression.compile(scope)?);
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
        let value = scope.new_anon_reg().init_from_node_output(scope, *init);

        let index = scope.new_anon_reg().init_from_const(
            scope,
            i64::try_from(index + 1)
                .map_err(|_| CompileError::TooManyTableEntries {
                    max: i64::MAX as usize,
                })?
                .into(),
        );

        scope.emit(opcodes::SetProperty::from((table, index, value)));
    }

    if let Some(last) = last {
        match last {
            NodeOutput::ReturnValues => {
                scope.emit(opcodes::SetAllPropertiesFromRet::from((
                    table,
                    initializers.len() + 1,
                )));
            }
            NodeOutput::VAStack => {
                scope.emit(opcodes::SetAllPropertiesFromVa::from((
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
