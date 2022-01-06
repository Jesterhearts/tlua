use indexmap::IndexSet;
use tlua_parser::ast::constant_string::ConstantString;

use crate::{
    encoding::{
        encode,
        ConstantTag,
        InstructionTag,
    },
    Constant,
    Register,
};

pub trait Encode<'a> {
    type Metadata;

    fn encode(&self, buffer: &'a mut Vec<u8>, metadata: Self::Metadata);
}

impl Encode<'_> for usize {
    type Metadata = ();

    fn encode(&self, buffer: &mut Vec<u8>, _: Self::Metadata) {
        encode(self, buffer);
    }
}

impl Encode<'_> for Register {
    type Metadata = ();

    fn encode(&self, buffer: &mut Vec<u8>, _: ()) {
        encode(self, buffer);
    }
}

impl<'a> Encode<'a> for Constant {
    type Metadata = &'a mut IndexSet<ConstantString>;

    fn encode(&self, buffer: &'a mut Vec<u8>, strings: Self::Metadata) {
        match *self {
            Constant::Nil => usize::from(ConstantTag::Nil).encode(buffer, ()),
            Constant::Bool(b) => {
                usize::from(ConstantTag::Bool).encode(buffer, ());
                usize::from(b).encode(buffer, ());
            }
            Constant::Float(f) => {
                usize::from(ConstantTag::Float).encode(buffer, ());
                let bits = f.to_le_bytes();
                encode(&bits, buffer);
            }
            Constant::Integer(i) => {
                usize::from(ConstantTag::Integer).encode(buffer, ());
                let bits = i.to_le_bytes();
                encode(&bits, buffer);
            }
            Constant::String(s) => {
                usize::from(ConstantTag::String).encode(buffer, ());
                let (tag, _) = strings.insert_full(s);
                tag.encode(buffer, ());
            }
        }
    }
}

impl Encode<'_> for (InstructionTag, Register, Register) {
    type Metadata = ();

    fn encode(&self, buffer: &mut Vec<u8>, _: ()) {
        let (tag, l, r) = self;
        usize::from(*tag).encode(buffer, ());
        l.encode(buffer, ());
        r.encode(buffer, ());
    }
}

impl<'a> Encode<'a> for (InstructionTag, Register, Constant) {
    type Metadata = <Constant as Encode<'a>>::Metadata;

    fn encode(&self, buffer: &'a mut Vec<u8>, metadata: Self::Metadata) {
        let (tag, l, r) = self;
        usize::from(*tag).encode(buffer, ());
        l.encode(buffer, ());
        r.encode(buffer, metadata);
    }
}
