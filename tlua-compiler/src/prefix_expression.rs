use either::Either;
use tlua_bytecode::{
    opcodes,
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
        unasm::UnasmRegister,
        InitRegister,
    },
    expressions::tables,
    CompileError,
    CompileExpression,
    CompileStatement,
    CompilerContext,
    NodeOutput,
};

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
        match self {
            VarPrefixExpression::Name(ident) => {
                compiler.read_variable(*ident).map(NodeOutput::Register)
            }
            VarPrefixExpression::TableAccess { head, middle, last } => {
                let src_reg = match emit_table_path_traversal(compiler, head, middle.iter())? {
                    Either::Left(reg) => reg,
                    Either::Right(err) => return Ok(NodeOutput::Err(err)),
                };

                match last {
                    VarAtom::Name(ident) => {
                        compiler.load_from_table(src_reg, ConstantString::from(ident))
                    }
                    VarAtom::IndexOp(index) => compiler.load_from_table(src_reg, index),
                }
                .map(|err| {
                    err.map(NodeOutput::Err)
                        .unwrap_or(NodeOutput::Register(src_reg))
                })
            }
        }
    }
}

impl CompileExpression for FnCallPrefixExpression<'_> {
    fn compile(&self, compiler: &mut CompilerContext) -> Result<NodeOutput, CompileError> {
        match self {
            FnCallPrefixExpression::Call { head, args } => {
                let target = match emit_load_head(compiler, head)? {
                    Either::Left(reg) => reg,
                    Either::Right(err) => return Ok(NodeOutput::Err(err)),
                };

                if let Some(err) = emit_call(compiler, target, args)? {
                    return Ok(NodeOutput::Err(err));
                }
            }
            FnCallPrefixExpression::CallPath { head, middle, last } => {
                let src_reg = match emit_table_path_traversal(compiler, head, middle.iter())? {
                    Either::Left(reg) => reg,
                    Either::Right(err) => return Ok(NodeOutput::Err(err)),
                };

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
) -> Result<Either<UnasmRegister, OpError>, CompileError> {
    match head {
        HeadAtom::Name(ident) => {
            let reg = NodeOutput::Register(compiler.read_variable(*ident)?);
            Ok(Either::Left(compiler.write_move_to_reg(reg).into()))
        }
        HeadAtom::Parenthesized(expr) => match expr.compile(compiler)? {
            NodeOutput::Constant(c) => {
                return Ok(Either::Right(compiler.write_raise(OpError::NotATable {
                    ty: c.short_type_name(),
                })))
            }
            NodeOutput::Err(err) => Ok(Either::Right(err)),
            src => Ok(Either::Left(compiler.write_move_to_reg(src).into())),
        },
    }
}

fn emit_table_path_traversal<'a, 'p>(
    compiler: &mut CompilerContext,
    head: &HeadAtom,
    middle: impl Iterator<Item = &'a PrefixAtom<'p>>,
) -> Result<Either<UnasmRegister, OpError>, CompileError>
where
    'p: 'a,
{
    let src_reg = match emit_load_head(compiler, head)? {
        Either::Left(reg) => reg,
        Either::Right(err) => return Ok(Either::Right(err)),
    };

    for next in middle {
        if let Some(err) = match next {
            PrefixAtom::Var(v) => compiler.load_from_table(src_reg, v)?,
            PrefixAtom::Function(atom) => emit_call(compiler, src_reg, atom)?,
        } {
            return Ok(Either::Right(err));
        };
    }
    Ok(Either::Left(src_reg))
}

fn emit_call(
    compiler: &mut CompilerContext,
    target: UnasmRegister,
    atom: &FunctionAtom,
) -> Result<Option<OpError>, CompileError> {
    Ok(match atom {
        FunctionAtom::Call(args) => emit_call_with_args(compiler, target, args)?,
        FunctionAtom::MethodCall { name: _, args: _ } => todo!(),
    })
}

fn emit_call_with_args(
    compiler: &mut CompilerContext,
    target: UnasmRegister,
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

fn emit_standard_call(
    compiler: &mut CompilerContext,
    target: UnasmRegister,
    mut args: impl ExactSizeIterator<Item = impl CompileExpression>,
) -> Result<Option<OpError>, CompileError> {
    let argc = args.len();
    if argc == 0 {
        // No arguments, just call.
        compiler.emit(opcodes::Call::from((target, 0, 0)));
        return Ok(None);
    }

    let (first_reg_idx, mut arg_registers) = compiler.new_anon_reg_range(argc);

    let regular_argc = argc - 1;

    for _ in 0..regular_argc {
        let arg_reg = arg_registers
            .next()
            .expect("Should still have arg registers");

        match args
            .next()
            .expect("Still in bounds for args")
            .compile(compiler)?
        {
            NodeOutput::Constant(c) => arg_reg.init_from_const(compiler, c),
            NodeOutput::Register(r) => arg_reg.init_from_reg(compiler, r),
            NodeOutput::ReturnValues => arg_reg.init_from_ret(compiler),
            NodeOutput::VAStack => arg_reg.init_from_va(compiler, 0),
            NodeOutput::Err(err) => return Ok(Some(err)),
        };
    }

    let last_reg = arg_registers.last().expect("Should have at least 1 arg");

    // Process the last argument in the list
    match args
        .next()
        .expect("Still in bounds args")
        .compile(compiler)?
    {
        NodeOutput::Constant(c) => {
            last_reg.init_from_const(compiler, c);
        }
        NodeOutput::Register(r) => {
            last_reg.init_from_reg(compiler, r);
        }
        NodeOutput::ReturnValues => {
            compiler.emit(opcodes::CallCopyRet::from((
                target,
                first_reg_idx,
                regular_argc,
            )));
            return Ok(None);
        }
        NodeOutput::VAStack => {
            compiler.emit(opcodes::CallCopyVa::from((
                target,
                first_reg_idx,
                regular_argc,
            )));
            return Ok(None);
        }
        NodeOutput::Err(err) => return Ok(Some(err)),
    }

    compiler.emit(opcodes::Call::from((target, first_reg_idx, argc)));
    Ok(None)
}
