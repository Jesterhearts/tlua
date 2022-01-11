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
    FuncId,
    Number,
    OpError,
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
    /// Allocate a new function
    AllocFunc(AllocFunc<RegisterTy>),
    /// Allocate a new function
    AllocTable(AllocTable<RegisterTy>),
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
    /// Load the target function as the current call target and begin mapping
    /// values into its registers. Extra arguments will populate the
    /// function's variadic argument list. Missing arguments will be cleared
    /// to nil.
    StartCall(StartCall<RegisterTy>),
    /// Performs the same operations as startcall, but allows for the inclusion
    /// of the most recent function's return values in its argument list.
    ///
    /// Specifically, if the last instruction was a call invocation (e.g.
    /// `DoCall` or `MapVarArgsAndDoCall`) the return values from that function
    /// exection will be appended to the list of arguments immediately before
    /// calling the target of this instruction
    StartCallExtending(StartCallExtending<RegisterTy>),
    /// Execute the function loaded by StartCall.
    DoCall,
    /// Copy this function's varargs into registers/varargs for the current call
    /// target and then begin executing it.
    MapVarArgsAndDoCall,
    /// Load the target value into the the next register for the current call
    /// target.
    MapArg(MapArg<RegisterTy>),
    /// Copy the first va arg into the next register for the current call target
    MapVa0,
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
pub struct StartCall<RegTy> {
    pub target: AnyReg<RegTy>,
    pub mapped_args: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, From)]
pub struct StartCallExtending<RegTy> {
    pub target: AnyReg<RegTy>,
    pub mapped_args: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, From)]
pub struct MapArg<RegTy> {
    pub src: Operand<RegTy>,
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
pub struct AllocFunc<RegTy> {
    pub dest: AnyReg<RegTy>,
    pub id: FuncId,
}

#[derive(Debug, Clone, Copy, PartialEq, From)]
pub struct AllocTable<RegTy> {
    pub dest: AnyReg<RegTy>,
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
