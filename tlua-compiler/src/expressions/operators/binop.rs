use scopeguard::guard_on_success;
use tlua_bytecode::{
    self,
    binop::{
        traits::{
            BooleanOpEval,
            ComparisonOpEval,
            NumericOpEval,
        },
        *,
    },
    Constant,
    ImmediateRegister,
    OpError,
};
use tlua_parser::{
    expressions::{
        operator,
        Expression,
    },
    StringTable,
};

use crate::{
    compiler::unasm::UnasmOp,
    CompileError,
    CompileExpression,
    NodeOutput,
    Scope,
};

pub(crate) fn write_binop<Op, Lhs, Rhs, ConstEval>(
    scope: &mut Scope,
    lhs: Lhs,
    rhs: Rhs,
    consteval: ConstEval,
) -> Result<NodeOutput, CompileError>
where
    Op: From<(ImmediateRegister, ImmediateRegister)> + Into<UnasmOp>,
    Lhs: CompileExpression,
    Rhs: CompileExpression,
    ConstEval: FnOnce(&StringTable, Constant, Constant) -> Result<Constant, OpError>,
{
    let lhs = lhs.compile(scope)?;
    let rhs = rhs.compile(scope)?;

    match (lhs, rhs) {
        (NodeOutput::Constant(lhs), NodeOutput::Constant(rhs)) => {
            match consteval(scope.string_table(), lhs, rhs) {
                Ok(constant) => Ok(NodeOutput::Constant(constant)),
                Err(err) => Ok(NodeOutput::Err(scope.write_raise(err))),
            }
        }
        (lhs, rhs) => {
            let lhs = lhs.into_register(scope);

            let rhs = rhs.into_register(scope);
            let mut scope = guard_on_success(scope, |scope| scope.pop_immediate(rhs));
            scope.emit(Op::from((lhs, rhs)));

            Ok(NodeOutput::Immediate(lhs))
        }
    }
}

fn write_numeric_binop<Op>(
    scope: &mut Scope,
    lhs: &Expression,
    rhs: &Expression,
) -> Result<NodeOutput, CompileError>
where
    Op: NumericOpEval + From<(ImmediateRegister, ImmediateRegister)> + Into<UnasmOp>,
{
    write_binop::<Op, _, _, _>(scope, lhs, rhs, |_, lhs, rhs| {
        Op::evaluate(lhs, rhs).map(|num| num.into())
    })
}

fn write_cmp_binop<Op>(
    scope: &mut Scope,
    lhs: &Expression,
    rhs: &Expression,
) -> Result<NodeOutput, CompileError>
where
    Op: ComparisonOpEval + From<(ImmediateRegister, ImmediateRegister)> + Into<UnasmOp>,
{
    write_binop::<Op, _, _, _>(scope, lhs, rhs, |strings, lhs, rhs| match (lhs, rhs) {
        (Constant::Nil, Constant::Nil) => Op::apply_nils().map(Constant::from),
        (Constant::Bool(lhs), Constant::Bool(rhs)) => Op::apply_bools(lhs, rhs).map(Constant::from),
        (Constant::Float(lhs), Constant::Float(rhs)) => {
            Ok(Op::apply_numbers(lhs.into(), rhs.into()).into())
        }
        (Constant::Float(lhs), Constant::Integer(rhs)) => {
            Ok(Op::apply_numbers(lhs.into(), rhs.into()).into())
        }
        (Constant::Integer(lhs), Constant::Integer(rhs)) => {
            Ok(Op::apply_numbers(lhs.into(), rhs.into()).into())
        }
        (Constant::Integer(lhs), Constant::Float(rhs)) => {
            Ok(Op::apply_numbers(lhs.into(), rhs.into()).into())
        }
        (Constant::String(lhs), Constant::String(rhs)) => Ok(Op::apply_strings(
            strings.get_string(lhs).expect("Valid string id"),
            strings.get_string(rhs).expect("Valid string id"),
        )
        .into()),
        // TODO(lang-5.4): This should be truthy for eq/ne.
        (lhs, rhs) => Err(OpError::CmpErr {
            lhs: lhs.short_type_name(),
            rhs: rhs.short_type_name(),
        }),
    })
}

fn write_boolean_binop<Op>(
    scope: &mut Scope,
    lhs: &Expression,
    rhs: &Expression,
) -> Result<NodeOutput, CompileError>
where
    Op: BooleanOpEval + From<(ImmediateRegister, ImmediateRegister)> + Into<UnasmOp>,
{
    write_binop::<Op, _, _, _>(scope, lhs, rhs, |_, lhs, rhs| Ok(Op::evaluate(lhs, rhs)))
}

impl CompileExpression for operator::Plus<'_> {
    fn compile(&self, scope: &mut Scope) -> Result<NodeOutput, CompileError> {
        write_numeric_binop::<Add>(scope, self.lhs, self.rhs)
    }
}

impl CompileExpression for operator::Minus<'_> {
    fn compile(&self, scope: &mut Scope) -> Result<NodeOutput, CompileError> {
        write_numeric_binop::<Subtract>(scope, self.lhs, self.rhs)
    }
}

impl CompileExpression for operator::Times<'_> {
    fn compile(&self, scope: &mut Scope) -> Result<NodeOutput, CompileError> {
        write_numeric_binop::<Times>(scope, self.lhs, self.rhs)
    }
}

impl CompileExpression for operator::Divide<'_> {
    fn compile(&self, scope: &mut Scope) -> Result<NodeOutput, CompileError> {
        write_numeric_binop::<Divide>(scope, self.lhs, self.rhs)
    }
}

impl CompileExpression for operator::IDiv<'_> {
    fn compile(&self, scope: &mut Scope) -> Result<NodeOutput, CompileError> {
        write_numeric_binop::<IDiv>(scope, self.lhs, self.rhs)
    }
}

impl CompileExpression for operator::Modulo<'_> {
    fn compile(&self, scope: &mut Scope) -> Result<NodeOutput, CompileError> {
        write_numeric_binop::<Modulo>(scope, self.lhs, self.rhs)
    }
}

impl CompileExpression for operator::Exponetiation<'_> {
    fn compile(&self, scope: &mut Scope) -> Result<NodeOutput, CompileError> {
        write_numeric_binop::<Exponetiation>(scope, self.lhs, self.rhs)
    }
}

impl CompileExpression for operator::BitAnd<'_> {
    fn compile(&self, scope: &mut Scope) -> Result<NodeOutput, CompileError> {
        write_numeric_binop::<BitAnd>(scope, self.lhs, self.rhs)
    }
}

impl CompileExpression for operator::BitOr<'_> {
    fn compile(&self, scope: &mut Scope) -> Result<NodeOutput, CompileError> {
        write_numeric_binop::<BitOr>(scope, self.lhs, self.rhs)
    }
}

impl CompileExpression for operator::BitXor<'_> {
    fn compile(&self, scope: &mut Scope) -> Result<NodeOutput, CompileError> {
        write_numeric_binop::<BitXor>(scope, self.lhs, self.rhs)
    }
}

impl CompileExpression for operator::ShiftLeft<'_> {
    fn compile(&self, scope: &mut Scope) -> Result<NodeOutput, CompileError> {
        write_numeric_binop::<ShiftLeft>(scope, self.lhs, self.rhs)
    }
}

impl CompileExpression for operator::ShiftRight<'_> {
    fn compile(&self, scope: &mut Scope) -> Result<NodeOutput, CompileError> {
        write_numeric_binop::<ShiftRight>(scope, self.lhs, self.rhs)
    }
}

impl CompileExpression for operator::Concat<'_> {
    fn compile(&self, scope: &mut Scope) -> Result<NodeOutput, CompileError> {
        let lhs = self.lhs.compile(scope)?;
        let rhs = self.rhs.compile(scope)?;

        let lhs = lhs.into_register(scope);

        let rhs = rhs.into_register(scope);
        let mut scope = guard_on_success(scope, |scope| scope.pop_immediate(rhs));
        scope.emit(Concat::from((lhs, rhs)));

        Ok(NodeOutput::Immediate(lhs))
    }
}

impl CompileExpression for operator::LessThan<'_> {
    fn compile(&self, scope: &mut Scope) -> Result<NodeOutput, CompileError> {
        write_cmp_binop::<LessThan>(scope, self.lhs, self.rhs)
    }
}

impl CompileExpression for operator::LessEqual<'_> {
    fn compile(&self, scope: &mut Scope) -> Result<NodeOutput, CompileError> {
        write_cmp_binop::<LessEqual>(scope, self.lhs, self.rhs)
    }
}

impl CompileExpression for operator::GreaterThan<'_> {
    fn compile(&self, scope: &mut Scope) -> Result<NodeOutput, CompileError> {
        write_cmp_binop::<GreaterThan>(scope, self.lhs, self.rhs)
    }
}

impl CompileExpression for operator::GreaterEqual<'_> {
    fn compile(&self, scope: &mut Scope) -> Result<NodeOutput, CompileError> {
        write_cmp_binop::<GreaterEqual>(scope, self.lhs, self.rhs)
    }
}

impl CompileExpression for operator::Equals<'_> {
    fn compile(&self, scope: &mut Scope) -> Result<NodeOutput, CompileError> {
        write_cmp_binop::<Equals>(scope, self.lhs, self.rhs)
    }
}

impl CompileExpression for operator::NotEqual<'_> {
    fn compile(&self, scope: &mut Scope) -> Result<NodeOutput, CompileError> {
        write_cmp_binop::<NotEqual>(scope, self.lhs, self.rhs)
    }
}

impl CompileExpression for operator::And<'_> {
    fn compile(&self, scope: &mut Scope) -> Result<NodeOutput, CompileError> {
        write_boolean_binop::<And>(scope, self.lhs, self.rhs)
    }
}

impl CompileExpression for operator::Or<'_> {
    fn compile(&self, scope: &mut Scope) -> Result<NodeOutput, CompileError> {
        write_boolean_binop::<Or>(scope, self.lhs, self.rhs)
    }
}
