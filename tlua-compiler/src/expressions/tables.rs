use scopeguard::guard_on_success;
use tlua_bytecode::{
    opcodes,
    ImmediateRegister,
    OpError,
};
use tlua_parser::expressions::{
    strings::ConstantString,
    tables::{
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
                let value = value.into_register(&mut scope);

                let mut scope = guard_on_success(&mut scope, |scope| scope.pop_immediate(value));

                scope.emit(opcodes::SetProperty::from((table, index, value)));

                last_field_va = false;
            }
            Field::Indexed {
                index: index_expr,
                expression,
            } => {
                let index_init = index_expr.compile(&mut scope)?;
                let index_init = index_init.into_register(&mut scope);
                let mut scope =
                    guard_on_success(&mut scope, |scope| scope.pop_immediate(index_init));

                index.init_from_immediate(&mut scope, index_init);

                let value = expression.compile(&mut scope)?;
                let value = value.into_register(&mut scope);
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

    let va_start = arraylike.len();
    let (last, initializers) = if last_field_va {
        (arraylike.pop(), arraylike)
    } else {
        (None, arraylike)
    };

    for (array_index, init) in initializers.into_iter().enumerate() {
        let value = init.into_register(&mut scope);
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
                scope.emit(opcodes::SetAllPropertiesFromRet::from((table, va_start)));
            }
            NodeOutput::VAStack => {
                scope.emit(opcodes::SetAllPropertiesFromVa::from((table, va_start)));
            }
            _ => {
                unreachable!("Only VA and return value nodes need special handling.")
            }
        }
    }

    Ok(None)
}
