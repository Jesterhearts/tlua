use tlua_parser::ast::{
    expressions::Expression,
    prefix_expression::{
        function_calls::FnArgs,
        *,
    },
};

use crate::{
    compiling::{
        CompileError,
        CompileExpression,
        CompileStatement,
        CompilerContext,
        NodeOutput,
    },
    vm::OpError,
};

impl CompileExpression for VarPrefixExpression<'_> {
    fn compile(&self, compiler: &mut CompilerContext) -> Result<NodeOutput, CompileError> {
        match self {
            VarPrefixExpression::Name(ident) => {
                let dest = compiler.read_variable(*ident)?;
                Ok(NodeOutput::Register(dest))
            }
            VarPrefixExpression::TableAccess {
                head: _,
                middle: _,
                last: _,
            } => todo!(),
        }
    }
}

impl CompileExpression for FnCallPrefixExpression<'_> {
    fn compile(&self, compiler: &mut CompilerContext) -> Result<NodeOutput, CompileError> {
        match self {
            FnCallPrefixExpression::Call { head, args } => match (head, args) {
                (HeadAtom::Name(ident), FunctionAtom::Call(args)) => {
                    let target = compiler.read_variable(*ident)?;
                    if let Some(err) = match args {
                        FnArgs::Expressions(exprs) => {
                            { compiler.write_call(target, exprs.iter()) }?
                        }
                        FnArgs::TableConstructor(_) => todo!(),
                        FnArgs::String(s) => {
                            compiler.write_call(target, std::iter::once(Expression::String(*s)))?
                        }
                    } {
                        return Ok(NodeOutput::Err(err));
                    }
                }
                (HeadAtom::Parenthesized(_), FunctionAtom::Call(_)) => todo!(),
                (HeadAtom::Name(_), FunctionAtom::MethodCall { name: _, args: _ }) => todo!(),
                (HeadAtom::Parenthesized(_), FunctionAtom::MethodCall { name: _, args: _ }) => {
                    todo!()
                }
            },
            FnCallPrefixExpression::CallPath {
                head: _,
                middle: _,
                last: _,
            } => todo!(),
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
