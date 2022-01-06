use tlua_parser::ast::constant_string::ConstantString;

use crate::{
    encoding::{
        decode,
        ConstantTag,
        InstructionTag,
    },
    Constant,
    Register,
};

pub trait Decode<'a> {
    type Data;
    type Metadata;
    fn decode(buffer: &'a [u8], metadata: Self::Metadata) -> (Self::Data, &[u8]);
}

impl Decode<'_> for InstructionTag {
    type Data = Self;
    type Metadata = ();

    fn decode(buffer: &[u8], _: ()) -> (Self::Data, &[u8]) {
        let (tag, buffer) = usize::decode(buffer, ());
        (InstructionTag::from(tag), buffer)
    }
}

impl Decode<'_> for ConstantTag {
    type Data = Self;
    type Metadata = ();

    fn decode(buffer: &[u8], _: ()) -> (Self::Data, &[u8]) {
        let (tag, buffer) = usize::decode(buffer, ());
        (Self::from(tag), buffer)
    }
}

impl Decode<'_> for usize {
    type Data = Self;
    type Metadata = ();

    fn decode(buffer: &[u8], _: ()) -> (Self::Data, &[u8]) {
        let (data, buffer) = decode(buffer);
        (*data, buffer)
    }
}

impl Decode<'_> for bool {
    type Data = bool;
    type Metadata = ();

    fn decode(buffer: &[u8], _: ()) -> (Self::Data, &[u8]) {
        let (word, buffer) = usize::decode(buffer, ());
        (word != 0, buffer)
    }
}

impl Decode<'_> for f64 {
    type Data = Self;
    type Metadata = ();

    fn decode(buffer: &[u8], _: ()) -> (Self::Data, &[u8]) {
        let (data, buffer) = decode(buffer);
        (f64::from_le_bytes(*data), buffer)
    }
}

impl Decode<'_> for i64 {
    type Data = Self;
    type Metadata = ();

    fn decode(buffer: &[u8], _: ()) -> (Self::Data, &[u8]) {
        let (data, buffer) = decode(buffer);
        (i64::from_le_bytes(*data), buffer)
    }
}

impl<'a> Decode<'a> for Register {
    type Data = Self;
    type Metadata = ();

    fn decode(buffer: &'a [u8], _: ()) -> (Self::Data, &[u8]) {
        let (data, buffer) = decode(buffer);
        (*data, buffer)
    }
}

impl<'a> Decode<'a> for ConstantString {
    type Data = Self;
    type Metadata = &'a Vec<ConstantString>;

    fn decode(buffer: &'a [u8], metadata: Self::Metadata) -> (Self::Data, &[u8]) {
        let (index, buffer) = usize::decode(buffer, ());
        (metadata[index], buffer)
    }
}

impl<'a> Decode<'a> for Constant {
    type Data = Self;
    type Metadata = &'a Vec<ConstantString>;

    fn decode(buffer: &'a [u8], metadata: Self::Metadata) -> (Self::Data, &[u8]) {
        let (tag, buffer) = ConstantTag::decode(buffer, ());
        match tag {
            ConstantTag::Bool => {
                let (data, buffer) = bool::decode(buffer, ());
                (Constant::Bool(data), buffer)
            }
            ConstantTag::Float => {
                let (data, buffer) = f64::decode(buffer, ());
                (Constant::Float(data), buffer)
            }
            ConstantTag::Integer => {
                let (data, buffer) = i64::decode(buffer, ());
                (Constant::Integer(data), buffer)
            }
            ConstantTag::String => {
                let (data, buffer) = ConstantString::decode(buffer, metadata);
                (Constant::String(data), buffer)
            }
            ConstantTag::Nil => (Constant::Nil, buffer),
        }
    }
}

impl Decode<'_> for (Register, Register) {
    type Data = Self;
    type Metadata = ();

    fn decode(buffer: &[u8], _: Self::Metadata) -> (Self::Data, &[u8]) {
        let (l, buffer) = decode(buffer);
        let (r, buffer) = decode(buffer);
        ((*l, *r), buffer)
    }
}

impl<'a> Decode<'a> for (Register, Constant) {
    type Data = Self;
    type Metadata = <Constant as Decode<'a>>::Metadata;

    fn decode(buffer: &'a [u8], metadata: Self::Metadata) -> (Self::Data, &[u8]) {
        let (l, buffer) = decode(buffer);
        let (r, buffer) = Constant::decode(buffer, metadata);
        ((*l, r), buffer)
    }
}
