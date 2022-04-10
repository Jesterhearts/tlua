use tlua_bytecode::{
    opcodes::*,
    MappedRegister,
    Register,
};

use crate::{
    Function,
    Instructions,
};

pub(crate) trait AssembleOp {
    type Target;

    fn assemble(self) -> Self::Target;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub(crate) struct OffsetRegister {
    pub(super) source_scope_depth: u16,
    pub(super) offset: u16,
}

impl From<OffsetRegister> for Register {
    fn from(val: OffsetRegister) -> Self {
        Register {
            scope: val.source_scope_depth,
            offset: val.offset,
        }
    }
}

pub(crate) type LocalRegister = OffsetRegister;

#[must_use]
pub(crate) type MappedLocalRegister = MappedRegister<LocalRegister>;

impl AssembleOp for MappedRegister<LocalRegister> {
    type Target = MappedRegister<Register>;

    fn assemble(self) -> Self::Target {
        Register {
            scope: self.source_scope_depth,
            offset: self.offset,
        }
        .into()
    }
}

pub(crate) type UnasmOp = Op<LocalRegister>;

impl AssembleOp for UnasmOp {
    type Target = Instruction;

    fn assemble(self) -> Self::Target {
        match self {
            Op::Nop => Op::Nop,
            Op::Add(op) => op.into(),
            Op::Subtract(op) => op.into(),
            Op::Times(op) => op.into(),
            Op::Modulo(op) => op.into(),
            Op::Divide(op) => op.into(),
            Op::Exponetiation(op) => op.into(),
            Op::IDiv(op) => op.into(),
            Op::BitAnd(op) => op.into(),
            Op::BitOr(op) => op.into(),
            Op::BitXor(op) => op.into(),
            Op::ShiftLeft(op) => op.into(),
            Op::ShiftRight(op) => op.into(),
            Op::UnaryMinus(op) => op.into(),
            Op::UnaryBitNot(op) => op.into(),
            Op::Not(op) => op.into(),
            Op::LessThan(op) => op.into(),
            Op::LessEqual(op) => op.into(),
            Op::GreaterThan(op) => op.into(),
            Op::GreaterEqual(op) => op.into(),
            Op::Equals(op) => op.into(),
            Op::NotEqual(op) => op.into(),
            Op::And(op) => op.into(),
            Op::Or(op) => op.into(),
            Op::Concat(op) => op.into(),
            Op::Length(op) => op.into(),
            Op::Raise(op) => op.into(),
            Op::RaiseIfNot(op) => op.into(),
            Op::Jump(op) => op.into(),
            Op::JumpNot(op) => op.into(),
            Op::JumpNil(op) => op.into(),
            Op::Lookup(op) => op.into(),
            Op::SetProperty(op) => op.into(),
            Op::SetAllPropertiesFromVa(op) => op.into(),
            Op::LoadConstant(op) => op.into(),
            Op::LoadRegister(LoadRegister { dst, src }) => LoadRegister {
                dst,
                src: src.assemble(),
            }
            .into(),
            Op::DuplicateRegister(op) => op.into(),
            Op::LoadVa(op) => op.into(),
            Op::Store(Store { dst, src }) => Store {
                dst: dst.assemble(),
                src,
            }
            .into(),
            Op::Alloc(op) => op.into(),
            Op::CheckType(op) => op.into(),
            Op::SetRet(op) => op.into(),
            Op::CopyRetFromVaAndRet => Op::CopyRetFromVaAndRet,
            Op::Ret => Op::Ret,
            Op::PushScope(op) => op.into(),
            Op::PopScope => Op::PopScope,
            Op::Call(op) => op.into(),
            Op::CallCopyRet(op) => op.into(),
            Op::CallCopyVa(op) => op.into(),
            Op::CopyRetFromRetAndRet => Op::CopyRetFromRetAndRet,
            Op::ConsumeRetRange(op) => op.into(),
            Op::SetAllPropertiesFromRet(op) => op.into(),
        }
    }
}

#[derive(Debug, Default, Clone)]
pub(crate) struct UnasmFunction {
    pub(crate) named_args: usize,
    pub(crate) immediates: usize,
    pub(crate) local_registers: usize,
    pub(crate) instructions: Vec<UnasmOp>,
}

impl UnasmFunction {
    pub(crate) fn into_function(self) -> Function {
        let Self {
            instructions,
            named_args,
            local_registers,
            immediates,
        } = self;

        Function {
            local_registers,
            immediates,
            named_args,
            instructions: Instructions::from(
                instructions
                    .into_iter()
                    .map(UnasmOp::assemble)
                    .collect::<Vec<_>>(),
            ),
        }
    }
}
