use scopeguard::guard_on_success;
use tlua_bytecode::{
    opcodes,
    ImmediateRegister,
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
        let table = scope.push_immediate().init_alloc_table(scope);

        emit_init_sequence(scope, table, self.fields.iter())?;

        Ok(NodeOutput::Immediate(table))
    }
}

pub(crate) fn emit_init_sequence<'a, 'f>(
    scope: &mut Scope,
    table: ImmediateRegister,
    fields: impl Iterator<Item = &'a Field<'f>>,
) -> Result<Option<OpError>, CompileError>
where
    'f: 'a,
{
    let mut arraylike = vec![];
    let mut last_field_va = false;

    let index = scope.push_immediate().no_init_needed();
    let mut scope = guard_on_success(scope, |scope| scope.pop_immediate(index));

    for field in fields {
        match field {
            Field::Named { name, expression } => {
                index.init_from_const(&mut scope, ConstantString::from(name).into());

                let value = expression.compile(&mut scope)?;
                let value = value.to_register(&mut scope);

                let mut scope = guard_on_success(&mut scope, |scope| scope.pop_immediate(value));

                scope.emit(opcodes::SetProperty::from((table, index, value)));

                last_field_va = false;
            }
            Field::Indexed {
                index: index_expr,
                expression,
            } => {
                let index_init = index_expr.compile(&mut scope)?;
                let index_init = index_init.to_register(&mut scope);
                let mut scope =
                    guard_on_success(&mut scope, |scope| scope.pop_immediate(index_init));

                index.init_from_immediate(&mut scope, index_init);

                let value = expression.compile(&mut scope)?;
                let value = value.to_register(&mut scope);
                let mut scope = guard_on_success(&mut scope, |scope| scope.pop_immediate(value));

                scope.emit(opcodes::SetProperty::from((table, index, value)));

                last_field_va = false;
            }
            Field::Arraylike { expression } => {
                arraylike.push(expression.compile(&mut scope)?);
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

    for (array_index, init) in initializers.iter().enumerate() {
        let value = init.to_register(&mut scope);
        let mut scope = guard_on_success(&mut scope, |scope| scope.pop_immediate(value));

        index.init_from_const(
            &mut scope,
            i64::try_from(array_index + 1)
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
