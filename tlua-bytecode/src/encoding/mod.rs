#![allow(dead_code)]

use std::fmt::Debug;

use bytemuck::Pod;
use num_enum::{
    FromPrimitive,
    IntoPrimitive,
};

use crate::{
    binop::{
        traits::FloatBinop,
        *,
    },
    encoding::decode::Decode,
    opcodes::*,
    ByteCodeError,
    Register,
};

pub mod decode;
pub mod encode;

pub trait Encodeable: Sized + Pod {
    const SIZE: usize = std::mem::size_of::<Self>();
    const ALIGN: usize = std::mem::align_of::<Self>();
}

impl<T> Encodeable for T where T: Pod {}

pub trait EncodableInstruction {
    const TAG: InstructionTag;
}

fn encode<T: Encodeable>(data: &T, buffer: &mut Vec<u8>) {
    let pad = T::ALIGN
        - buffer
            .last()
            .map(|t| (t as *const u8 as usize))
            .unwrap_or(0);
    for _ in 0..pad {
        buffer.push(0);
    }

    let bytes = bytemuck::bytes_of(data);
    buffer.extend_from_slice(bytes);
}

fn decode<T: Encodeable>(mut buffer: &[u8]) -> (&T, &[u8]) {
    let pad = buffer.first().unwrap() as *const u8 as usize % T::ALIGN;
    buffer = buffer.split_at(pad).1;

    let (data, buffer) = buffer.split_at(T::SIZE);
    (bytemuck::from_bytes(data), buffer)
}

impl<OpTy, LhsTy, RhsTy> From<FloatOp<OpTy, LhsTy, RhsTy>> for (InstructionTag, LhsTy, RhsTy)
where
    OpTy: FloatBinop + EncodableInstruction,
{
    fn from(val: FloatOp<OpTy, LhsTy, RhsTy>) -> Self {
        (OpTy::TAG, val.lhs, val.rhs)
    }
}

#[derive(Clone)]
pub struct InstructionStream(Vec<u8>);

impl InstructionStream {
    fn iter(&self) -> InstructionIter {
        InstructionIter {
            data: self.0.as_slice(),
        }
    }
}

pub struct InstructionIter<'a> {
    data: &'a [u8],
}

impl Iterator for InstructionIter<'_> {
    type Item = Result<Instruction, ByteCodeError>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.data.is_empty() {
            return None;
        }

        // TODO(cleanup): Use destructuring assignment when stable & available.
        let (tag, buf) = InstructionTag::decode(self.data, ());
        self.data = buf;

        match tag {
            InstructionTag::UnaryMinus => {
                let (reg, buf) = Register::decode(self.data, ());
                self.data = buf;
                Some(Ok(UnaryMinus::from(reg).into()))
            }
            InstructionTag::Not => todo!(),
            InstructionTag::UnaryBitNot => todo!(),
            InstructionTag::Length => todo!(),
            InstructionTag::Add => todo!(),
            InstructionTag::Subtract => todo!(),
            InstructionTag::Times => todo!(),
            InstructionTag::Modulo => todo!(),
            InstructionTag::Divide => todo!(),
            InstructionTag::IDiv => todo!(),
            InstructionTag::BitAnd => todo!(),
            InstructionTag::BitOr => todo!(),
            InstructionTag::BitXor => todo!(),
            InstructionTag::ShiftLeft => todo!(),
            InstructionTag::ShiftRight => todo!(),
            InstructionTag::And => todo!(),
            InstructionTag::Or => todo!(),
            InstructionTag::LessThan => todo!(),
            InstructionTag::LessEqual => todo!(),
            InstructionTag::GreaterThan => todo!(),
            InstructionTag::GreaterEqual => todo!(),
            InstructionTag::Equals => todo!(),
            InstructionTag::NotEqual => todo!(),
            InstructionTag::Concat => todo!(),
            InstructionTag::SubtractIndirect => todo!(),
            InstructionTag::AddIndirect => todo!(),
            InstructionTag::TimesIndirect => todo!(),
            InstructionTag::ModuloIndirect => todo!(),
            InstructionTag::DivideIndirect => todo!(),
            InstructionTag::Exponetiation => todo!(),
            InstructionTag::ExponetiationIndirect => todo!(),
            InstructionTag::IDivIndirect => todo!(),
            InstructionTag::BitAndIndirect => todo!(),
            InstructionTag::BitOrIndirect => todo!(),
            InstructionTag::BitXorIndirect => todo!(),
            InstructionTag::ShiftLeftIndirect => todo!(),
            InstructionTag::ShiftRightIndirect => todo!(),
            InstructionTag::AndIndirect => todo!(),
            InstructionTag::OrIndirect => todo!(),
            InstructionTag::LessThanIndirect => todo!(),
            InstructionTag::LessEqualIndirect => todo!(),
            InstructionTag::GreaterThanIndirect => todo!(),
            InstructionTag::GreaterEqualIndirect => todo!(),
            InstructionTag::EqualsIndirect => todo!(),
            InstructionTag::NotEqualIndirect => todo!(),
            InstructionTag::ConcatIndirect => todo!(),
            InstructionTag::Set => todo!(),
            InstructionTag::SetIndirect => todo!(),
            InstructionTag::SetFromVa => todo!(),
            InstructionTag::Jump => todo!(),
            InstructionTag::JumpNot => todo!(),
            InstructionTag::JumpNotRet0 => todo!(),
            InstructionTag::JumpNotVa0 => todo!(),
            InstructionTag::Load => todo!(),
            InstructionTag::LoadIndirect => todo!(),
            InstructionTag::Store => todo!(),
            InstructionTag::StoreConstant => todo!(),
            InstructionTag::StoreFromVa => todo!(),
            InstructionTag::StoreIndirect => todo!(),
            InstructionTag::StoreConstantIndirect => todo!(),
            InstructionTag::StoreFromVaIndirect => todo!(),
            InstructionTag::StoreAllFromVa => todo!(),
            InstructionTag::AllocFunc => todo!(),
            InstructionTag::AllocTable => todo!(),
            InstructionTag::PushScope => todo!(),
            InstructionTag::PopScope => todo!(),
            InstructionTag::Raise => todo!(),
            InstructionTag::Ret => todo!(),
            InstructionTag::StartCall => todo!(),
            InstructionTag::StartCallExtending => todo!(),
            InstructionTag::InvalidInstruction => todo!(),
        }
    }
}

impl Debug for InstructionStream {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut list = f.debug_list();
        for isn in self.iter() {
            match isn {
                Ok(o) => list.entry(&o),
                Err(e) => list.entry(&format_args!("invalid instruction {:?}", e)),
            };
        }
        list.finish()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, FromPrimitive, IntoPrimitive)]
#[repr(usize)]
pub enum ConstantTag {
    Bool = 0,
    Float,
    Integer,
    String,
    #[default]
    Nil,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, FromPrimitive, IntoPrimitive)]
#[repr(usize)]
pub enum InstructionTag {
    // Unary operations.
    UnaryMinus = 0,
    Not,
    UnaryBitNot,
    Length,
    // Reg + Constant binary operations.
    Add,
    Subtract,
    Times,
    Modulo,
    Divide,
    IDiv,
    BitAnd,
    BitOr,
    BitXor,
    ShiftLeft,
    ShiftRight,
    And,
    Or,
    LessThan,
    LessEqual,
    GreaterThan,
    GreaterEqual,
    Equals,
    NotEqual,
    Concat,
    // Reg + Reg binary operations.
    SubtractIndirect,
    AddIndirect,
    TimesIndirect,
    ModuloIndirect,
    DivideIndirect,
    Exponetiation,
    ExponetiationIndirect,
    IDivIndirect,
    BitAndIndirect,
    BitOrIndirect,
    BitXorIndirect,
    ShiftLeftIndirect,
    ShiftRightIndirect,
    AndIndirect,
    OrIndirect,
    LessThanIndirect,
    LessEqualIndirect,
    GreaterThanIndirect,
    GreaterEqualIndirect,
    EqualsIndirect,
    NotEqualIndirect,
    ConcatIndirect,
    // Register operations.
    Set,
    SetIndirect,
    SetFromVa,
    // Control flow
    Jump,
    JumpNot,
    JumpNotRet0,
    JumpNotVa0,
    // Table operations
    Load,
    LoadIndirect,
    Store,
    StoreConstant,
    StoreFromVa,
    StoreIndirect,
    StoreConstantIndirect,
    StoreFromVaIndirect,
    StoreAllFromVa,
    // Allocation
    AllocFunc,
    AllocTable,
    PushScope,
    PopScope,

    // Exit the function
    Raise,
    Ret,

    // Start a function call
    StartCall,
    StartCallExtending,

    #[default]
    InvalidInstruction,
}

#[derive(Debug, PartialEq, Eq, FromPrimitive, IntoPrimitive)]
#[repr(usize)]
pub enum StartCallInstructionTag {
    // Arg mapping
    MapArg = InstructionTag::InvalidInstruction as usize,
    MapArgIndirect,
    MapVa0,

    // Invoke call
    DoCall,
    MapVarArgsAndDoCall,

    #[default]
    InvalidInstruction,
}

#[derive(Debug, PartialEq, Eq, FromPrimitive, IntoPrimitive)]
#[repr(usize)]
pub enum MapRetInstructionTag {
    SetRet = StartCallInstructionTag::InvalidInstruction as usize,
    SetRetIndirect,
    SetRetVa0,
    SetRetFromRet0,
    MapRet,
    CopyRetFromRetAndRet,
    CopyRetFromVaAndRet,

    #[default]
    InvalidInstruction,
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;

    use crate::{
        binop::{
            Add,
            FloatOp,
        },
        encoding::{
            decode::Decode,
            encode::Encode,
            InstructionTag,
        },
        Register,
    };

    #[test]
    fn encode_decode_add() {
        let isn = FloatOp::<Add, _, _>::from((
            Register {
                scope: None,
                offset: 0,
            },
            Register {
                scope: None,
                offset: 1,
            },
        ));
        let mut buf = vec![];
        <(_, _, _)>::from(isn).encode(&mut buf, ());

        let (tag, rest) = InstructionTag::decode(buf.as_slice(), ());
        assert_eq!(tag, InstructionTag::Add);

        let (data, _) = <(Register, Register)>::decode(rest, ());
        let op = FloatOp::<Add, _, _>::from(data);

        assert_eq!(op.lhs, isn.lhs);
        assert_eq!(op.rhs, isn.rhs);
    }
}
