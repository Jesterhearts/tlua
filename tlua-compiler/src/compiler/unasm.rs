use tlua_bytecode::{
    opcodes::*,
    MappedRegister,
    Register,
};

use crate::Function;

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

pub(crate) type MappedLocalRegister = MappedRegister<LocalRegister>;
pub(crate) type UnasmRegister = AnyReg<LocalRegister>;
pub(crate) type UnasmOperand = Operand<LocalRegister>;

impl From<OffsetRegister> for UnasmRegister {
    fn from(reg: OffsetRegister) -> Self {
        Self::Register(reg.into())
    }
}

pub(crate) type UnasmOp = Op<LocalRegister>;

impl AssembleOp for AnyReg<LocalRegister> {
    type Target = AnyReg<Register>;

    fn assemble(self) -> Self::Target {
        match self {
            AnyReg::Register(r) => AnyReg::from(MappedRegister::from(Register::from(*r))),
            AnyReg::Immediate(i) => i.into(),
        }
    }
}

impl AssembleOp for Operand<LocalRegister> {
    type Target = Operand<Register>;

    fn assemble(self) -> Self::Target {
        match self {
            Operand::Nil => Operand::Nil,
            Operand::Bool(c) => Operand::Bool(c),
            Operand::Float(c) => Operand::Float(c),
            Operand::Integer(c) => Operand::Integer(c),
            Operand::String(c) => Operand::String(c),
            Operand::Register(c) => Operand::Register(MappedRegister::from(Register::from(*c))),
            Operand::Immediate(c) => Operand::Immediate(c),
        }
    }
}

impl AssembleOp for (UnasmRegister, Operand<LocalRegister>) {
    type Target = (AnyReg<Register>, Operand<Register>);

    fn assemble(self) -> Self::Target {
        let (lhs, rhs) = self;
        (lhs.assemble(), rhs.assemble())
    }
}

impl AssembleOp for (UnasmRegister, UnasmRegister) {
    type Target = (AnyReg<Register>, Operand<Register>);

    fn assemble(self) -> Self::Target {
        let (lhs, rhs) = self;
        (lhs.assemble(), rhs.into())
    }
}

impl AssembleOp for UnasmOp {
    type Target = Instruction;

    fn assemble(self) -> Self::Target {
        match self {
            Op::Nop => Op::Nop,
            Op::Add(op) => Op::Add(<(_, _)>::from(op).assemble().into()),
            Op::Subtract(op) => Op::Subtract(<(_, _)>::from(op).assemble().into()),
            Op::Times(op) => Op::Times(<(_, _)>::from(op).assemble().into()),
            Op::Modulo(op) => Op::Modulo(<(_, _)>::from(op).assemble().into()),
            Op::Divide(op) => Op::Divide(<(_, _)>::from(op).assemble().into()),
            Op::Exponetiation(op) => Op::Exponetiation(<(_, _)>::from(op).assemble().into()),
            Op::IDiv(op) => Op::IDiv(<(_, _)>::from(op).assemble().into()),
            Op::BitAnd(op) => Op::BitAnd(<(_, _)>::from(op).assemble().into()),
            Op::BitOr(op) => Op::BitOr(<(_, _)>::from(op).assemble().into()),
            Op::BitXor(op) => Op::BitXor(<(_, _)>::from(op).assemble().into()),
            Op::ShiftLeft(op) => Op::ShiftLeft(<(_, _)>::from(op).assemble().into()),
            Op::ShiftRight(op) => Op::ShiftRight(<(_, _)>::from(op).assemble().into()),
            Op::UnaryMinus(UnaryMinus { reg }) => UnaryMinus {
                reg: reg.assemble(),
            }
            .into(),
            Op::Not(Not { reg }) => Not {
                reg: reg.assemble(),
            }
            .into(),
            Op::UnaryBitNot(UnaryBitNot { reg }) => UnaryBitNot {
                reg: reg.assemble(),
            }
            .into(),
            Op::LessThan(op) => Op::LessThan(<(_, _)>::from(op).assemble().into()),
            Op::LessEqual(op) => Op::LessEqual(<(_, _)>::from(op).assemble().into()),
            Op::GreaterThan(op) => Op::GreaterThan(<(_, _)>::from(op).assemble().into()),
            Op::GreaterEqual(op) => Op::GreaterEqual(<(_, _)>::from(op).assemble().into()),
            Op::Equals(op) => Op::Equals(<(_, _)>::from(op).assemble().into()),
            Op::NotEqual(op) => Op::NotEqual(<(_, _)>::from(op).assemble().into()),
            Op::And(op) => Op::And(<(_, _)>::from(op).assemble().into()),
            Op::Or(op) => Op::Or(<(_, _)>::from(op).assemble().into()),
            Op::Concat(op) => Op::Concat(<(_, _)>::from(op).assemble().into()),
            Op::Length(Length { reg }) => Length {
                reg: reg.assemble(),
            }
            .into(),
            Op::Raise(op) => op.into(),
            Op::Jump(op) => op.into(),
            Op::JumpNot(JumpNot { cond, target }) => JumpNot {
                cond: cond.assemble(),
                target,
            }
            .into(),
            Op::JumpNotRet0(op) => op.into(),
            Op::JumpNotVa0(op) => op.into(),
            Op::Load(Load { dest, index }) => Load {
                dest: dest.assemble(),
                index: index.assemble(),
            }
            .into(),
            Op::Store(Store { dest, src, index }) => Store {
                dest: dest.assemble(),
                src: src.assemble(),
                index: index.assemble(),
            }
            .into(),
            Op::StoreFromVa(StoreFromVa {
                dest,
                va_index,
                index,
            }) => StoreFromVa {
                dest: dest.assemble(),
                va_index,
                index: index.assemble(),
            }
            .into(),
            Op::StoreAllFromVa(StoreAllFromVa { dest, start_index }) => StoreAllFromVa {
                dest: dest.assemble(),
                start_index,
            }
            .into(),
            Op::Set(Set { dest, source }) => Set {
                dest: dest.assemble(),
                source: source.assemble(),
            }
            .into(),
            Op::SetFromVa(SetFromVa { dest, index }) => SetFromVa {
                dest: dest.assemble(),
                index,
            }
            .into(),
            Op::Alloc(Alloc {
                dest,
                type_id,
                metadata,
            }) => Alloc {
                dest: dest.assemble(),
                type_id,
                metadata,
            }
            .into(),
            Op::PushScope(descriptor) => descriptor.into(),
            Op::PopScope => Op::PopScope,
            Op::Call(Call {
                target,
                mapped_args_start,
                mapped_args_count,
            }) => Call {
                target: target.assemble(),
                mapped_args_start,
                mapped_args_count,
            }
            .into(),
            Op::CallCopyRet(CallCopyRet {
                target,
                mapped_args_start,
                mapped_args_count,
            }) => CallCopyRet {
                target: target.assemble(),
                mapped_args_start,
                mapped_args_count,
            }
            .into(),
            Op::CallCopyVa(CallCopyVa {
                target,
                mapped_args_start,
                mapped_args_count,
            }) => CallCopyVa {
                target: target.assemble(),
                mapped_args_start,
                mapped_args_count,
            }
            .into(),
            Op::SetRet(SetRet { src }) => SetRet {
                src: src.assemble(),
            }
            .into(),
            Op::SetRetVa0 => Op::SetRetVa0,
            Op::SetRetFromRet0 => Op::SetRetFromRet0,
            Op::CopyRetFromRetAndRet => Op::CopyRetFromRetAndRet,
            Op::CopyRetFromVaAndRet => Op::CopyRetFromVaAndRet,
            Op::Ret => Op::Ret,
            Op::MapRet(MapRet { dest }) => MapRet {
                dest: dest.assemble(),
            }
            .into(),
            Op::StoreRet(StoreRet { dest, index }) => StoreRet {
                dest: dest.assemble(),
                index: index.assemble(),
            }
            .into(),
            Op::StoreAllRet(StoreAllRet { dest, start_index }) => StoreAllRet {
                dest: dest.assemble(),
                start_index,
            }
            .into(),
        }
    }
}

#[derive(Debug, Default, Clone)]
pub(crate) struct UnasmFunction {
    pub(crate) named_args: usize,
    pub(crate) anon_registers: usize,
    pub(crate) local_registers: usize,
    pub(crate) instructions: Vec<UnasmOp>,
}

impl UnasmFunction {
    pub(crate) fn into_function(self) -> Function {
        let Self {
            instructions,
            named_args,
            local_registers,
            anon_registers,
        } = self;

        Function {
            local_registers,
            anon_registers,
            named_args,
            instructions: instructions.into_iter().map(UnasmOp::assemble).collect(),
        }
    }
}
