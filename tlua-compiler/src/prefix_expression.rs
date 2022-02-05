use either::Either;
use tlua_bytecode::{
    opcodes,
    AnonymousRegister,
    OpError,
};
use tlua_parser::ast::{
    constant_string::ConstantString,
    expressions::Expression,
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
    expressions::tables,
    CompileError,
    CompileExpression,
    CompileStatement,
    CompilerContext,
    NodeOutput,
};

pub(crate) struct TableIndex {
    pub(crate) table: AnonymousRegister,
    pub(crate) index: AnonymousRegister,
}

impl CompileExpression for VarAtom<'_> {
    fn compile(&self, compiler: &mut CompilerContext) -> Result<NodeOutput, CompileError> {
        match self {
            VarAtom::Name(ident) => ConstantString::from(ident).compile(compiler),
            VarAtom::IndexOp(index) => index.compile(compiler),
        }
    }
}

impl CompileExpression for VarPrefixExpression<'_> {
    fn compile(&self, compiler: &mut CompilerContext) -> Result<NodeOutput, CompileError> {
        map_var(compiler, self).map(|out| match out {
            Either::Left(reg) => NodeOutput::MappedRegister(reg),
            Either::Right(TableIndex { table, index }) => NodeOutput::TableEntry { table, index },
        })
    }
}

impl CompileExpression for FnCallPrefixExpression<'_> {
    fn compile(&self, compiler: &mut CompilerContext) -> Result<NodeOutput, CompileError> {
        match self {
            FnCallPrefixExpression::Call { head, args } => {
                let target = emit_load_head(compiler, head)?;

                if let Some(err) = emit_call(compiler, target, args)? {
                    return Ok(NodeOutput::Err(err));
                }
            }
            FnCallPrefixExpression::CallPath { head, middle, last } => {
                let src_reg = emit_table_path_traversal(compiler, head, middle.iter())?;

                if let Some(err) = emit_call(compiler, src_reg, last)? {
                    return Ok(NodeOutput::Err(err));
                };
            }
        };

        Ok(NodeOutput::ReturnValues)
    }
}

impl CompileStatement for FnCallPrefixExpression<'_> {
    fn compile(&self, compiler: &mut CompilerContext) -> Result<Option<OpError>, CompileError> {
        match CompileExpression::compile(&self, compiler)? {
            NodeOutput::Err(err) => Ok(Some(err)),
            _ => Ok(None),
        }
    }
}

fn emit_load_head(
    compiler: &mut CompilerContext,
    head: &HeadAtom,
) -> Result<AnonymousRegister, CompileError> {
    match head {
        HeadAtom::Name(ident) => {
            let reg = compiler.read_variable(*ident)?;
            Ok(compiler.new_anon_reg().init_from_mapped_reg(compiler, reg))
        }
        HeadAtom::Parenthesized(expr) => match expr.compile(compiler)? {
            NodeOutput::Constant(c) => {
                compiler.write_raise(OpError::NotATable {
                    ty: c.short_type_name(),
                });
                Ok(compiler.new_anon_reg().no_init_needed())
            }
            NodeOutput::Err(_) => Ok(compiler.new_anon_reg().no_init_needed()),
            src => Ok(compiler.output_to_reg_reuse_anon(src)),
        },
    }
}

fn emit_table_path_traversal<'a, 'p>(
    compiler: &mut CompilerContext,
    head: &HeadAtom,
    middle: impl Iterator<Item = &'a PrefixAtom<'p>>,
) -> Result<AnonymousRegister, CompileError>
where
    'p: 'a,
{
    let table_reg = emit_load_head(compiler, head)?;

    for next in middle {
        match next {
            PrefixAtom::Var(v) => {
                let index = v.compile(compiler)?;
                let index = compiler.output_to_reg_reuse_anon(index);
                compiler.emit(opcodes::Lookup::from((table_reg, table_reg, index)));
            }
            PrefixAtom::Function(atom) => {
                emit_call(compiler, table_reg, atom)?;
            }
        };
    }

    Ok(table_reg)
}

fn emit_call(
    compiler: &mut CompilerContext,
    target: AnonymousRegister,
    atom: &FunctionAtom,
) -> Result<Option<OpError>, CompileError> {
    Ok(match atom {
        FunctionAtom::Call(args) => emit_call_with_args(compiler, target, args)?,
        FunctionAtom::MethodCall { name: _, args: _ } => todo!(),
    })
}

fn emit_call_with_args(
    compiler: &mut CompilerContext,
    target: AnonymousRegister,
    args: &FnArgs,
) -> Result<Option<OpError>, CompileError> {
    Ok(match args {
        FnArgs::Expressions(exprs) => emit_standard_call(compiler, target, exprs.iter())?,
        FnArgs::TableConstructor(ctor) => {
            tables::emit_init_sequence(compiler, target, ctor.fields.iter())?
        }
        FnArgs::String(s) => {
            emit_standard_call(compiler, target, std::iter::once(Expression::String(*s)))?
        }
    })
}

pub(crate) fn emit_standard_call(
    compiler: &mut CompilerContext,
    target: AnonymousRegister,
    mut args: impl ExactSizeIterator<Item = impl CompileExpression>,
) -> Result<Option<OpError>, CompileError> {
    let argc = args.len();
    if argc == 0 {
        // No arguments, just call.
        compiler.emit(opcodes::Call::from((target, 0, 0)));
        return Ok(None);
    }

    let mut arg_registers = compiler.new_anon_reg_range(argc);
    let first_arg_idx = usize::from(
        arg_registers
            .clone()
            .next()
            .expect("At least one arg.")
            .no_init_needed(),
    );

    let regular_argc = argc - 1;

    for _ in 0..regular_argc {
        let arg_reg = arg_registers
            .next()
            .expect("Should still have arg registers");

        let arg_init = args
            .next()
            .expect("Still in bounds for args")
            .compile(compiler)?;

        arg_reg.init_from_node_output(compiler, arg_init);
    }

    let last_reg = arg_registers.last().expect("Should have at least 1 arg");

    // Process the last argument in the list
    match args
        .next()
        .expect("Still in bounds args")
        .compile(compiler)?
    {
        NodeOutput::ReturnValues => {
            compiler.emit(opcodes::CallCopyRet::from((
                target,
                first_arg_idx,
                regular_argc,
            )));
            return Ok(None);
        }
        NodeOutput::VAStack => {
            compiler.emit(opcodes::CallCopyVa::from((
                target,
                first_arg_idx,
                regular_argc,
            )));
            return Ok(None);
        }
        arg => {
            last_reg.init_from_node_output(compiler, arg);
        }
    }

    compiler.emit(opcodes::Call::from((target, first_arg_idx, argc)));
    Ok(None)
}

pub(crate) fn map_var(
    compiler: &mut CompilerContext,
    expr: &VarPrefixExpression,
) -> Result<Either<MappedLocalRegister, TableIndex>, CompileError> {
    match expr {
        VarPrefixExpression::Name(ident) => Ok(Either::Left(compiler.read_variable(*ident)?)),
        VarPrefixExpression::TableAccess { head, middle, last } => {
            let table = emit_table_path_traversal(compiler, head, middle.iter())?;
            let index = match last {
                VarAtom::Name(ident) => compiler
                    .new_anon_reg()
                    .init_from_const(compiler, ConstantString::from(ident).into()),
                VarAtom::IndexOp(index) => {
                    let index = index.compile(compiler)?;
                    compiler.output_to_reg_reuse_anon(index)
                }
            };
            Ok(Either::Right(TableIndex { table, index }))
        }
    }
}
