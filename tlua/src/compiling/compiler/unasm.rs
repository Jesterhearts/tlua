use std::num::NonZeroU16;

use derive_more::{
    Deref,
    From,
};

use crate::{
    compiling::Function,
    vm::{
        binop::*,
        opcodes::{
            Instruction,
            Op,
            *,
        },
        Constant,
        Register,
    },
};

pub trait AssembleOp {
    type Target;

    fn assemble(self) -> Self::Target;
}

impl From<AnonymousRegister> for Register {
    fn from(val: AnonymousRegister) -> Self {
        Register {
            scope: None,
            offset: val.0,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct OffsetRegister {
    pub(super) source_scope: u16,
    pub(super) offset: u16,
}

impl From<OffsetRegister> for Register {
    fn from(val: OffsetRegister) -> Self {
        Register {
            scope: NonZeroU16::new(val.source_scope + 1),
            offset: val.offset,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Deref)]
pub struct ConstantRegister(pub(super) OffsetRegister);

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, From)]
pub enum LocalRegister {
    Mutable(OffsetRegister),
    Constant(ConstantRegister),
}

impl From<LocalRegister> for Register {
    fn from(val: LocalRegister) -> Self {
        match val {
            LocalRegister::Mutable(m) => m.into(),
            LocalRegister::Constant(c) => c.0.into(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, From)]
pub struct AnonymousRegister(u16);

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, From)]
pub enum UnasmRegister {
    Anonymous(AnonymousRegister),
    Local(LocalRegister),
}

impl From<OffsetRegister> for UnasmRegister {
    fn from(reg: OffsetRegister) -> Self {
        Self::Local(LocalRegister::Mutable(reg))
    }
}

impl From<ConstantRegister> for UnasmRegister {
    fn from(reg: ConstantRegister) -> Self {
        Self::Local(LocalRegister::Constant(reg))
    }
}

impl From<UnasmRegister> for Register {
    fn from(val: UnasmRegister) -> Self {
        match val {
            UnasmRegister::Anonymous(anon) => anon.into(),
            UnasmRegister::Local(local) => local.into(),
        }
    }
}

pub type UnasmOp = Op<UnasmRegister>;

impl<OpTy> AssembleOp for BinOp<OpTy, UnasmRegister, Constant> {
    type Target = (Register, Constant);

    fn assemble(self) -> Self::Target {
        let data: (_, _) = self.into();
        data.assemble()
    }
}

impl<OpTy> AssembleOp for BinOp<OpTy, UnasmRegister, UnasmRegister> {
    type Target = (Register, Register);

    fn assemble(self) -> Self::Target {
        let data: (_, _) = self.into();
        data.assemble()
    }
}

impl AssembleOp for (UnasmRegister, Constant) {
    type Target = (Register, Constant);

    fn assemble(self) -> Self::Target {
        let (lhs, rhs) = self;
        (lhs.into(), rhs)
    }
}

impl AssembleOp for (UnasmRegister, UnasmRegister) {
    type Target = (Register, Register);

    fn assemble(self) -> Self::Target {
        let (lhs, rhs) = self;
        (lhs.into(), rhs.into())
    }
}

impl UnasmOp {
    fn assemble(self) -> Instruction {
        match self {
            Op::Add(op) => Add::from(op.assemble()).into(),
            Op::AddIndirect(op) => AddIndirect::from(op.assemble()).into(),
            Op::Subtract(op) => Subtract::from(op.assemble()).into(),
            Op::SubtractIndirect(op) => SubtractIndirect::from(op.assemble()).into(),
            Op::Times(op) => Times::from(op.assemble()).into(),
            Op::TimesIndirect(op) => TimesIndirect::from(op.assemble()).into(),
            Op::Modulo(op) => Modulo::from(op.assemble()).into(),
            Op::ModuloIndirect(op) => ModuloIndirect::from(op.assemble()).into(),
            Op::Divide(op) => Divide::from(op.assemble()).into(),
            Op::DivideIndirect(op) => DivideIndirect::from(op.assemble()).into(),
            Op::Exponetiation(op) => Exponetiation::from(op.assemble()).into(),
            Op::ExponetiationIndirect(op) => ExponetiationIndirect::from(op.assemble()).into(),
            Op::IDiv(op) => IDiv::from(op.assemble()).into(),
            Op::IDivIndirect(op) => IDivIndirect::from(op.assemble()).into(),
            Op::BitAnd(op) => BitAnd::from(op.assemble()).into(),
            Op::BitAndIndirect(op) => BitAndIndirect::from(op.assemble()).into(),
            Op::BitOr(op) => BitOr::from(op.assemble()).into(),
            Op::BitOrIndirect(op) => BitOrIndirect::from(op.assemble()).into(),
            Op::BitXor(op) => BitXor::from(op.assemble()).into(),
            Op::BitXorIndirect(op) => BitXorIndirect::from(op.assemble()).into(),
            Op::ShiftLeft(op) => ShiftLeft::from(op.assemble()).into(),
            Op::ShiftLeftIndirect(op) => ShiftLeftIndirect::from(op.assemble()).into(),
            Op::ShiftRight(op) => ShiftRight::from(op.assemble()).into(),
            Op::ShiftRightIndirect(op) => ShiftRightIndirect::from(op.assemble()).into(),
            Op::UnaryMinus(UnaryMinus { reg }) => UnaryMinus { reg: reg.into() }.into(),
            Op::Not(Not { reg }) => Not { reg: reg.into() }.into(),
            Op::UnaryBitNot(UnaryBitNot { reg }) => UnaryBitNot { reg: reg.into() }.into(),
            Op::LessThan(op) => LessThan::from(op.assemble()).into(),
            Op::LessThanIndirect(op) => LessThanIndirect::from(op.assemble()).into(),
            Op::LessEqual(op) => LessEqual::from(op.assemble()).into(),
            Op::LessEqualIndirect(op) => LessEqualIndirect::from(op.assemble()).into(),
            Op::GreaterThan(op) => GreaterThan::from(op.assemble()).into(),
            Op::GreaterThanIndirect(op) => GreaterThanIndirect::from(op.assemble()).into(),
            Op::GreaterEqual(op) => GreaterEqual::from(op.assemble()).into(),
            Op::GreaterEqualIndirect(op) => GreaterEqualIndirect::from(op.assemble()).into(),
            Op::Equals(op) => Equals::from(op.assemble()).into(),
            Op::EqualsIndirect(op) => EqualsIndirect::from(op.assemble()).into(),
            Op::NotEqual(op) => NotEqual::from(op.assemble()).into(),
            Op::NotEqualIndirect(op) => NotEqualIndirect::from(op.assemble()).into(),
            Op::And(op) => And::from(op.assemble()).into(),
            Op::AndIndirect(op) => AndIndirect::from(op.assemble()).into(),
            Op::Or(op) => Or::from(op.assemble()).into(),
            Op::OrIndirect(op) => OrIndirect::from(op.assemble()).into(),
            Op::Concat(op) => Concat::from(op.assemble()).into(),
            Op::ConcatIndirect(op) => ConcatIndirect::from(op.assemble()).into(),
            Op::Length(Length { reg }) => Length { reg: reg.into() }.into(),
            Op::Raise(op) => op.into(),
            Op::Jump(op) => op.into(),
            Op::JumpNot(JumpNot { cond, target }) => JumpNot {
                cond: cond.into(),
                target,
            }
            .into(),
            Op::JumpNotRet0(op) => op.into(),
            Op::JumpNotVa0(op) => op.into(),
            Op::Set(Set { dest, source }) => Set {
                dest: dest.into(),
                source,
            }
            .into(),
            Op::SetIndirect(SetIndirect { dest, source }) => SetIndirect {
                dest: dest.into(),
                source: source.into(),
            }
            .into(),
            Op::SetFromVa(SetFromVa { dest, index }) => SetFromVa {
                dest: dest.into(),
                index,
            }
            .into(),
            Op::AllocFunc(AllocFunc { dest, id }) => AllocFunc {
                dest: dest.into(),
                id,
            }
            .into(),
            Op::PushScope(descriptor) => descriptor.into(),
            Op::PopScope => Op::PopScope,
            Op::StartCall(StartCall { target }) => StartCall {
                target: target.into(),
            }
            .into(),
            Op::StartCallExtending(StartCallExtending { target }) => StartCallExtending {
                target: target.into(),
            }
            .into(),
            Op::DoCall => Op::DoCall,
            Op::MapVarArgsAndDoCall => Op::MapVarArgsAndDoCall,
            Op::MapArg(op) => op.into(),
            Op::MapArgIndirect(MapArgIndirect { src }) => MapArgIndirect { src: src.into() }.into(),
            Op::MapVa0 => Op::MapVa0,
            Op::SetRet(op) => op.into(),
            Op::SetRetIndirect(SetRetIndirect { src }) => SetRetIndirect { src: src.into() }.into(),
            Op::SetRetVa0 => Op::SetRetVa0,
            Op::SetRetFromRet0 => Op::SetRetFromRet0,
            Op::CopyRetFromRetAndRet => Op::CopyRetFromRetAndRet,
            Op::CopyRetFromVaAndRet => Op::CopyRetFromVaAndRet,
            Op::Ret => Op::Ret,
            Op::MapRet(MapRet { dest }) => MapRet { dest: dest.into() }.into(),
        }
    }
}

#[derive(Debug, Default, Clone)]
pub struct UnasmFunction {
    pub named_args: usize,
    pub anon_registers: usize,
    pub local_registers: usize,
    pub instructions: Vec<UnasmOp>,
}

impl UnasmFunction {
    pub fn into_function(self) -> Function {
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
            instructions: instructions.into_iter().map(Op::assemble).collect(),
        }
    }
}
