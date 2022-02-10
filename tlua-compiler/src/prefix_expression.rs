use either::Either;
use tlua_bytecode::{
    opcodes,
    AnonymousRegister,
    OpError,
};
use tlua_parser::ast::{
    constant_string::ConstantString,
    expressions::Expression,
    identifiers::Ident,
    prefix_expression::{
        function_calls::FnArgs,
        *,
    },
};

use crate::{
    compiler::{
        unasm::MappedLocalRegister,
        InitRegister,
    },
    constant::Constant,
    expressions::tables,
    CompileError,
    CompileExpression,
    CompileStatement,
    NodeOutput,
    Scope,
};

pub(crate) struct TableIndex {
    pub(crate) table: AnonymousRegister,
    pub(crate) index: AnonymousRegister,
}

impl CompileExpression for VarAtom<'_> {
    fn compile(&self, scope: &mut Scope) -> Result<NodeOutput, CompileError> {
        match self {
            VarAtom::Name(ident) => ConstantString::from(ident).compile(scope),
            VarAtom::IndexOp(index) => index.compile(scope),
        }
    }
}

impl CompileExpression for VarPrefixExpression<'_> {
    fn compile(&self, scope: &mut Scope) -> Result<NodeOutput, CompileError> {
        map_var(scope, self).map(|out| match out {
            Either::Left(reg) => NodeOutput::MappedRegister(reg),
            Either::Right(TableIndex { table, index }) => NodeOutput::TableEntry { table, index },
        })
    }
}

impl CompileExpression for FnCallPrefixExpression<'_> {
    fn compile(&self, scope: &mut Scope) -> Result<NodeOutput, CompileError> {
        match self {
            FnCallPrefixExpression::Call { head, args } => {
                let target = emit_load_head(scope, head)?;

                if let Some(err) = emit_call(scope, target, args)? {
                    return Ok(NodeOutput::Err(err));
                }
            }
            FnCallPrefixExpression::CallPath { head, middle, last } => {
                let src_reg = emit_table_path_traversal(scope, head, middle.iter())?;

                if let Some(err) = emit_call(scope, src_reg, last)? {
                    return Ok(NodeOutput::Err(err));
                };
            }
        };

        Ok(NodeOutput::ReturnValues)
    }
}

impl CompileStatement for FnCallPrefixExpression<'_> {
    fn compile(&self, scope: &mut Scope) -> Result<Option<OpError>, CompileError> {
        match CompileExpression::compile(&self, scope)? {
            NodeOutput::Err(err) => Ok(Some(err)),
            _ => Ok(None),
        }
    }
}

fn emit_load_head(scope: &mut Scope, head: &HeadAtom) -> Result<AnonymousRegister, CompileError> {
    match head {
        HeadAtom::Name(ident) => {
            let reg = scope.read_variable(*ident)?;
            Ok(scope.new_anon_reg().init_from_mapped_reg(scope, reg))
        }
        HeadAtom::Parenthesized(expr) => match expr.compile(scope)? {
            NodeOutput::Constant(c) => {
                scope.write_raise(OpError::NotATable {
                    ty: c.short_type_name(),
                });
                Ok(scope.new_anon_reg().no_init_needed())
            }
            NodeOutput::Err(_) => Ok(scope.new_anon_reg().no_init_needed()),
            src => Ok(scope.output_to_reg_reuse_anon(src)),
        },
    }
}

fn emit_table_path_traversal<'a, 'p>(
    scope: &mut Scope,
    head: &HeadAtom,
    middle: impl Iterator<Item = &'a PrefixAtom<'p>>,
) -> Result<AnonymousRegister, CompileError>
where
    'p: 'a,
{
    let table_reg = emit_load_head(scope, head)?;

    for next in middle {
        match next {
            PrefixAtom::Var(v) => {
                let index = v.compile(scope)?;
                let index = scope.output_to_reg_reuse_anon(index);
                scope.emit(opcodes::Lookup::from((table_reg, table_reg, index)));
            }
            PrefixAtom::Function(atom) => {
                emit_call(scope, table_reg, atom)?;
            }
        };
    }

    Ok(table_reg)
}

fn emit_call(
    scope: &mut Scope,
    target: AnonymousRegister,
    atom: &FunctionAtom,
) -> Result<Option<OpError>, CompileError> {
    Ok(match atom {
        FunctionAtom::Call(args) => emit_call_with_args(scope, target, None, args)?,
        FunctionAtom::MethodCall { name, args } => {
            emit_call_with_args(scope, target, Some(*name), args)?
        }
    })
}

fn emit_call_with_args(
    scope: &mut Scope,
    target: AnonymousRegister,
    method: Option<Ident>,
    args: &FnArgs,
) -> Result<Option<OpError>, CompileError> {
    Ok(match args {
        FnArgs::Expressions(exprs) => emit_standard_call(scope, target, method, exprs.iter())?,
        FnArgs::TableConstructor(ctor) => {
            tables::emit_init_sequence(scope, target, ctor.fields.iter())?
        }
        FnArgs::String(s) => emit_standard_call(
            scope,
            target,
            method,
            std::iter::once(Expression::String(*s)),
        )?,
    })
}

pub(crate) fn emit_standard_call(
    scope: &mut Scope,
    target: AnonymousRegister,
    method: Option<Ident>,
    mut args: impl ExactSizeIterator<Item = impl CompileExpression>,
) -> Result<Option<OpError>, CompileError> {
    let argc = args.len() + method.iter().len();
    if argc == 0 {
        // No arguments, just call.
        scope.emit(opcodes::Call::from((target, 0, 0)));
        return Ok(None);
    }

    let mut arg_registers = scope.new_anon_reg_range(argc).peekable();
    let first_arg_idx = usize::from(
        arg_registers
            .peek()
            .cloned()
            .expect("At least one arg.")
            .no_init_needed(),
    );

    if let Some(method) = method {
        let arg_reg = arg_registers
            .next()
            .expect("Should still have arg registers");

        arg_reg.init_from_anon_reg(scope, target);
        let index_reg = arg_registers
            .peek()
            .cloned()
            .unwrap_or_else(|| scope.new_anon_reg());

        let index_reg = index_reg.init_from_const(scope, Constant::String(method.into()));

        scope.emit(opcodes::Lookup::from((target, target, index_reg)));
    }

    for _ in 0..arg_registers.len() - 1 {
        let arg_reg = arg_registers.next().expect("Still in bounds of args");

        let arg_init = args
            .next()
            .expect("Still in bounds for args")
            .compile(scope)?;

        arg_reg.init_from_node_output(scope, arg_init);
    }

    let last_reg = arg_registers.next().expect("Should have at least 1 arg");

    // Process the last argument in the list
    match args
        .next()
        .expect("Still in bounds of args")
        .compile(scope)?
    {
        NodeOutput::ReturnValues => {
            scope.emit(opcodes::CallCopyRet::from((
                target,
                first_arg_idx,
                argc - 1,
            )));
            return Ok(None);
        }
        NodeOutput::VAStack => {
            scope.emit(opcodes::CallCopyVa::from((target, first_arg_idx, argc - 1)));
            return Ok(None);
        }
        arg => {
            last_reg.init_from_node_output(scope, arg);
        }
    }

    scope.emit(opcodes::Call::from((target, first_arg_idx, argc)));
    Ok(None)
}

pub(crate) fn map_var(
    scope: &mut Scope,
    expr: &VarPrefixExpression,
) -> Result<Either<MappedLocalRegister, TableIndex>, CompileError> {
    match expr {
        VarPrefixExpression::Name(ident) => Ok(Either::Left(scope.read_variable(*ident)?)),
        VarPrefixExpression::TableAccess { head, middle, last } => {
            let table = emit_table_path_traversal(scope, head, middle.iter())?;
            let index = match last {
                VarAtom::Name(ident) => scope
                    .new_anon_reg()
                    .init_from_const(scope, ConstantString::from(ident).into()),
                VarAtom::IndexOp(index) => {
                    let index = index.compile(scope)?;
                    scope.output_to_reg_reuse_anon(index)
                }
            };
            Ok(Either::Right(TableIndex { table, index }))
        }
    }
}
