use either::Either;
use scopeguard::guard_on_success;
use tlua_bytecode::{
    opcodes,
    ImmediateRegister,
    OpError,
};
use tlua_parser::{
    expressions::Expression,
    identifiers::Ident,
    prefix_expression::{
        function_calls::FnArgs,
        *,
    },
    string::ConstantString,
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
    pub(crate) table: ImmediateRegister,
    pub(crate) index: ImmediateRegister,
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
            NodeOutput::Immediate(imm) => {
                scope.pop_immediate(imm);
                Ok(None)
            }
            _ => Ok(None),
        }
    }
}

fn emit_load_head(scope: &mut Scope, head: &HeadAtom) -> Result<ImmediateRegister, CompileError> {
    match head {
        HeadAtom::Name(ident) => {
            let reg = scope.read_variable(*ident)?;
            Ok(scope.push_immediate().init_from_mapped_reg(scope, reg))
        }
        HeadAtom::Parenthesized(expr) => match expr.compile(scope)? {
            NodeOutput::Constant(c) => {
                scope.write_raise(OpError::NotATable {
                    ty: c.short_type_name(),
                });
                Ok(scope.push_immediate().no_init_needed())
            }
            NodeOutput::Err(_) => Ok(scope.push_immediate().no_init_needed()),
            src => Ok(src.into_register(scope)),
        },
    }
}

fn emit_table_path_traversal<'a, 'p>(
    scope: &mut Scope,
    head: &HeadAtom,
    middle: impl Iterator<Item = &'a PrefixAtom<'p>>,
) -> Result<ImmediateRegister, CompileError>
where
    'p: 'a,
{
    let table_reg = emit_load_head(scope, head)?;

    for next in middle {
        match next {
            PrefixAtom::Var(v) => {
                let index = v.compile(scope)?;
                let index = index.into_register(scope);
                let mut scope = guard_on_success(&mut *scope, |scope| scope.pop_immediate(index));

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
    target: ImmediateRegister,
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
    target: ImmediateRegister,
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
    target: ImmediateRegister,
    method: Option<Ident>,
    mut args: impl ExactSizeIterator<Item = impl CompileExpression>,
) -> Result<Option<OpError>, CompileError> {
    let argc = args.len() + method.iter().len();
    if argc == 0 {
        // No arguments, just call.
        scope.emit(opcodes::Call::from((target, 0, 0)));
        return Ok(None);
    }

    let arg_range = scope.reserve_immediate_range(argc);
    let mut arg_registers = arg_range.iter().peekable();
    let mut scope = guard_on_success(scope, |scope| scope.pop_immediate_range(arg_range));

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

        arg_reg.init_from_immediate(&mut scope, target);
        let index_reg = scope
            .push_immediate()
            .init_from_const(&mut scope, Constant::String(method.into()));
        let mut scope = guard_on_success(&mut scope, |scope| {
            scope.pop_immediate(index_reg);
        });

        scope.emit(opcodes::Lookup::from((target, target, index_reg)));
    }

    for _ in 0..arg_registers.len() - 1 {
        let arg_reg = arg_registers.next().expect("Still in bounds of args");

        let arg_init = args
            .next()
            .expect("Still in bounds for args")
            .compile(&mut scope)?;

        let arg_init = arg_init.into_register(&mut scope);
        let mut scope = guard_on_success(&mut scope, |scope| scope.pop_immediate(arg_init));

        arg_reg.init_from_immediate(&mut scope, arg_init);
    }

    let last_reg = arg_registers.next().expect("Should have at least 1 arg");

    // Process the last argument in the list
    match args
        .next()
        .expect("Still in bounds of args")
        .compile(&mut scope)?
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
            let arg_init = arg.into_register(&mut scope);
            let mut scope = guard_on_success(&mut scope, |scope| scope.pop_immediate(arg_init));

            last_reg.init_from_immediate(&mut scope, arg_init);
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
                    .push_immediate()
                    .init_from_const(scope, ConstantString::from(ident).into()),
                VarAtom::IndexOp(index) => index.compile(scope)?.into_register(scope),
            };

            Ok(Either::Right(TableIndex { table, index }))
        }
    }
}
