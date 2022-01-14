use derive_more::{
    From,
    Into,
};
use tlua_parser::ast::constant_string::ConstantString;

use crate::{
    binop::*,
    register::{
        AnonymousRegister,
        MappedRegister,
        Register,
    },
    Number,
    OpError,
    TypeId,
    TypeMeta,
};

#[derive(Debug, Clone, Copy, PartialEq, From)]
pub enum AnyReg<RegisterTy> {
    Register(MappedRegister<RegisterTy>),
    Immediate(AnonymousRegister),
}

#[derive(Debug, Clone, Copy, PartialEq, From)]
pub enum Operand<RegisterTy> {
    Nil,
    Bool(bool),
    Float(f64),
    Integer(i64),
    String(ConstantString),
    Register(MappedRegister<RegisterTy>),
    Immediate(AnonymousRegister),
}

impl<T, O> From<AnyReg<O>> for Operand<T>
where
    T: From<O>,
    O: Copy,
{
    fn from(a: AnyReg<O>) -> Self {
        match a {
            AnyReg::Register(r) => MappedRegister::from(T::from(*r)).into(),
            AnyReg::Immediate(i) => i.into(),
        }
    }
}

impl<T> From<Number> for Operand<T> {
    fn from(n: Number) -> Self {
        match n {
            Number::Float(f) => Self::Float(f),
            Number::Integer(i) => Self::Integer(i),
        }
    }
}

/// An opcode using the bytecode's representation of a register.
pub type Instruction = Op<Register>;

/// The full list of opcodes supported in tlua's bytecode format. This is
/// generic over the register type to allow intermediate forms of bytecode.
#[derive(Debug, Clone, Copy, PartialEq, From)]
pub enum Op<RegisterTy> {
    /// `[dest] += [src]`
    Add(FloatOp<Add, RegisterTy>),
    /// `[dest] -= [src]`, preserving types.
    Subtract(FloatOp<Subtract, RegisterTy>),
    /// `[dest] *= [src]`, preserving types.
    Times(FloatOp<Times, RegisterTy>),
    /// `[dest] %= [src]`, preserving types.
    Modulo(FloatOp<Modulo, RegisterTy>),
    /// `[dest] = [dest] / [src]`, producing a float.
    Divide(FloatOp<Divide, RegisterTy>),
    /// `[dest] = [dest].exp([src])`, producing a float.
    Exponetiation(FloatOp<Exponetiation, RegisterTy>),
    /// `[dest] = floor([dest] / [src])`, type preserving.
    IDiv(FloatOp<IDiv, RegisterTy>),
    /// `[dest] = [dest] & [src]`, producing an int.
    BitAnd(IntOp<BitAnd, RegisterTy>),
    /// `[dest] = [dest] | [src]`, producing an int.
    BitOr(IntOp<BitOr, RegisterTy>),
    /// `[dest] = [dest] ^ [src]`, producing an int.
    BitXor(IntOp<BitXor, RegisterTy>),
    /// `[dest] = [dest] << [src]`, producing an int.
    ShiftLeft(IntOp<ShiftLeft, RegisterTy>),
    /// `[dest] = [dest] >> [src]`, producing an int.
    ShiftRight(IntOp<ShiftRight, RegisterTy>),
    /// `[dest] = -[dest]`, type preserving.
    UnaryMinus(UnaryMinus<RegisterTy>),
    /// `[dest] = !([dest] as bool)`, producing a bool.
    Not(Not<RegisterTy>),
    /// `[dest] = ![dest]`, producing an int.
    UnaryBitNot(UnaryBitNot<RegisterTy>),
    /// `[dest] = [dest] < [src]`.
    LessThan(CompareOp<LessThan, RegisterTy>),
    /// `[dest] = [dest] <= [src]`.
    LessEqual(CompareOp<LessEqual, RegisterTy>),
    /// `[dest] = [dest] > [src]`.
    GreaterThan(CompareOp<GreaterThan, RegisterTy>),
    /// `[dest] = [dest] >= [src]`.
    GreaterEqual(CompareOp<GreaterEqual, RegisterTy>),
    /// `[dest] = [dest] == [src]`.
    Equals(CompareOp<Equals, RegisterTy>),
    /// `[dest] = [dest] != [src]`.
    NotEqual(CompareOp<NotEqual, RegisterTy>),
    /// `[dest] = [dest] as bool ? [src] : [dest]`.
    And(BoolOp<And, RegisterTy>),
    /// `[dest] = [dest] as bool ? [dest] : [src]`.
    Or(BoolOp<Or, RegisterTy>),
    /// `[dest] = [dest].to_string() + [src].to_string()`.
    Concat(Concat<RegisterTy>),
    /// `[dest] = [dest].len()`.
    Length(Length<RegisterTy>),
    /// Immediately return from the current function with a specific error.
    Raise(Raise),
    /// Unconditionally jump to the targt instruction
    Jump(Jump),
    /// Jump to a specific instruction if the value in the register evaluates to
    /// false.
    JumpNot(JumpNot<RegisterTy>),
    /// Jump to a specific instruction if the first return value evaluates to
    /// false.
    JumpNotRet0(JumpNotRet0),
    /// Jump to a specific instruction if the first variadic argument
    JumpNotVa0(JumpNotVa0),
    /// `[dest] = `[dest].table[[src]]`
    Load(Load<RegisterTy>),
    /// `[dest].table[[index]]` = `[src]`
    Store(Store<RegisterTy>),
    /// `[dest].table[[index]]` = `va[c]`
    StoreFromVa(StoreFromVa<RegisterTy>),
    /// `[dest].table[(start, ..)]` = `va...`
    StoreAllFromVa(StoreAllFromVa<RegisterTy>),
    /// Initialize a register from a value.
    Set(Set<RegisterTy>),
    /// Initialize a register from a variadic argument.
    SetFromVa(SetFromVa<RegisterTy>),
    /// Allocate a type
    Alloc(Alloc<RegisterTy>),
    /// Push a new scope as the current local scope.
    PushScope(ScopeDescriptor),
    /// Discard the current scope and restore the most recently pushed scope.
    PopScope,
    /// Copy the target register value into this function's output list.
    SetRet(SetRet<RegisterTy>),
    /// Copy the first va arg into this function's output list.
    SetRetVa0,
    /// Copy the first return value from a function into this function's output
    /// list.
    SetRetFromRet0,
    /// Copy all return values from this function's va list and then return from
    /// the function.
    CopyRetFromVaAndRet,
    /// Stop executing this function and return.
    Ret,
    /// Load the target function as the current call target and copy a range of
    /// anonymous register as that function's arguments.
    Call(Call<RegisterTy>),
    /// Performs the same operations as `Call` but maps the results of the most
    /// recent call into the target's arguments.
    CallCopyRet(CallCopyRet<RegisterTy>),
    /// Performs the same operations as `Call`, but maps the current list of
    /// variadic arguments into the target's arguments.
    CallCopyVa(CallCopyVa<RegisterTy>),
    /// Copy all return values from a function into this function's output list
    /// and then return from the function.
    CopyRetFromRetAndRet,
    /// Copy the next available return value into the target register.
    MapRet(MapRet<RegisterTy>),
    /// Copy the next available return value into the index loaded from a
    /// register into a table.
    StoreRet(StoreRet<RegisterTy>),
    /// Copy all the available return values into a table.
    StoreAllRet(StoreAllRet<RegisterTy>),
}

#[derive(Debug, Clone, Copy, PartialEq, From, Into)]
pub struct Concat<RegTy> {
    lhs: AnyReg<RegTy>,
    rhs: Operand<RegTy>,
}

#[derive(Debug, Clone, Copy, PartialEq, From)]
pub struct UnaryMinus<RegTy> {
    pub reg: AnyReg<RegTy>,
}

#[derive(Debug, Clone, Copy, PartialEq, From)]
pub struct UnaryBitNot<RegTy> {
    pub reg: AnyReg<RegTy>,
}

#[derive(Debug, Clone, Copy, PartialEq, From)]
pub struct Not<RegTy> {
    pub reg: AnyReg<RegTy>,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Length<RegTy> {
    pub reg: AnyReg<RegTy>,
}

#[derive(Debug, Clone, Copy, PartialEq, From)]
pub struct Jump {
    pub target: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, From)]
pub struct JumpNot<RegTy> {
    pub cond: AnyReg<RegTy>,
    pub target: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, From)]
pub struct JumpNotRet0 {
    pub target: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, From)]
pub struct JumpNotVa0 {
    pub target: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, From)]
pub struct Call<RegTy> {
    pub target: AnyReg<RegTy>,
    pub mapped_args_start: usize,
    pub mapped_args_count: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, From)]
pub struct CallCopyRet<RegTy> {
    pub target: AnyReg<RegTy>,
    pub mapped_args_start: usize,
    pub mapped_args_count: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, From)]
pub struct CallCopyVa<RegTy> {
    pub target: AnyReg<RegTy>,
    pub mapped_args_start: usize,
    pub mapped_args_count: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, From)]
pub struct SetRet<RegTy> {
    pub src: Operand<RegTy>,
}

#[derive(Debug, Clone, Copy, PartialEq, From)]
pub struct Raise {
    pub err: OpError,
}

#[derive(Debug, Clone, Copy, PartialEq, From)]
pub struct Alloc<RegTy> {
    pub dest: AnyReg<RegTy>,
    pub type_id: TypeId,
    pub metadata: TypeMeta,
}

#[derive(Debug, Clone, Copy, PartialEq, From)]
pub struct Store<RegTy> {
    pub dest: AnyReg<RegTy>,
    pub index: Operand<RegTy>,
    pub src: Operand<RegTy>,
}

#[derive(Debug, Clone, Copy, PartialEq, From)]
pub struct StoreFromVa<RegTy> {
    pub dest: AnyReg<RegTy>,
    pub index: Operand<RegTy>,
    pub va_index: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, From)]
pub struct StoreAllFromVa<RegTy> {
    pub dest: AnyReg<RegTy>,
    pub start_index: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, From)]
pub struct Load<RegTy> {
    pub dest: AnyReg<RegTy>,
    pub index: Operand<RegTy>,
}

#[derive(Debug, Clone, Copy, PartialEq, From)]
pub struct Set<RegTy> {
    pub dest: AnyReg<RegTy>,
    pub source: Operand<RegTy>,
}

#[derive(Debug, Clone, Copy, PartialEq, From)]
pub struct SetFromVa<RegTy> {
    pub dest: AnyReg<RegTy>,
    pub index: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, From)]
pub struct ScopeDescriptor {
    pub size: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, From)]
pub struct MapRet<RegTy> {
    pub dest: AnyReg<RegTy>,
}

#[derive(Debug, Clone, Copy, PartialEq, From)]
pub struct StoreRet<RegTy> {
    pub dest: AnyReg<RegTy>,
    pub index: Operand<RegTy>,
}

#[derive(Debug, Clone, Copy, PartialEq, From)]
pub struct StoreAllRet<RegTy> {
    pub dest: AnyReg<RegTy>,
    pub start_index: usize,
}
