use derive_more::{
    Deref,
    From,
};

use crate::{
    binop::*,
    constant::Constant,
    register::Register,
    FuncId,
    OpError,
};

/// An opcode using the bytecode's representation of a register.
pub type Instruction = Op<Register>;

/// The full list of opcodes supported in tlua's bytecode format. This is
/// generic over the register type to allow intermediate forms of bytecode.
#[derive(Debug, Clone, Copy, PartialEq, From)]
pub enum Op<RegisterTy> {
    /// `[dest] += c`, preserving types.
    Add(Add<RegisterTy, Constant>),
    /// `[dest] += [src]`
    AddIndirect(AddIndirect<RegisterTy, RegisterTy>),
    /// `[dest] -= c`, preserving types.
    Subtract(Subtract<RegisterTy, Constant>),
    /// `[dest] -= [src]`, preserving types.
    SubtractIndirect(SubtractIndirect<RegisterTy, RegisterTy>),
    /// `[dest] *= c`, preserving types.
    Times(Times<RegisterTy, Constant>),
    /// `[dest] *= [src]`, preserving types.
    TimesIndirect(TimesIndirect<RegisterTy, RegisterTy>),
    /// `[dest] %= c`, preserving types.
    Modulo(Modulo<RegisterTy, Constant>),
    /// `[dest] %= [src]`, preserving types.
    ModuloIndirect(ModuloIndirect<RegisterTy, RegisterTy>),
    /// `[dest] = [dest] / c`, producing a float.
    Divide(Divide<RegisterTy, Constant>),
    /// `[dest] = [dest] / [src]`, producing a float.
    DivideIndirect(DivideIndirect<RegisterTy, RegisterTy>),
    /// `[dest] = [dest].exp(c)`, producing a float.
    Exponetiation(Exponetiation<RegisterTy, Constant>),
    /// `[dest] = [dest].exp([src])`, producing a float.
    ExponetiationIndirect(ExponetiationIndirect<RegisterTy, RegisterTy>),
    /// `[dest] = floor([dest] / c)`, type preserving.
    IDiv(IDiv<RegisterTy, Constant>),
    /// `[dest] = floor([dest] / [src])`, type preserving.
    IDivIndirect(IDivIndirect<RegisterTy, RegisterTy>),
    /// `[dest] = [dest] & c`, producing an int.
    BitAnd(BitAnd<RegisterTy, Constant>),
    /// `[dest] = [dest] & [src]`, producing an int.
    BitAndIndirect(BitAndIndirect<RegisterTy, RegisterTy>),
    /// `[dest] = [dest] | c`, producing an int.
    BitOr(BitOr<RegisterTy, Constant>),
    /// `[dest] = [dest] | [src]`, producing an int.
    BitOrIndirect(BitOrIndirect<RegisterTy, RegisterTy>),
    /// `[dest] = [dest] ^ c`, producing an int.
    BitXor(BitXor<RegisterTy, Constant>),
    /// `[dest] = [dest] ^ [src]`, producing an int.
    BitXorIndirect(BitXorIndirect<RegisterTy, RegisterTy>),
    /// `[dest] = [dest] << c`, producing an int.
    ShiftLeft(ShiftLeft<RegisterTy, Constant>),
    /// `[dest] = [dest] << [src]`, producing an int.
    ShiftLeftIndirect(ShiftLeftIndirect<RegisterTy, RegisterTy>),
    /// `[dest] = [dest] >> c`, producing an int.
    ShiftRight(ShiftRight<RegisterTy, Constant>),
    /// `[dest] = [dest] >> [src]`, producing an int.
    ShiftRightIndirect(ShiftRightIndirect<RegisterTy, RegisterTy>),
    /// `[dest] = -[dest]`, type preserving.
    UnaryMinus(UnaryMinus<RegisterTy>),
    /// `[dest] = !([dest] as bool)`, producing a bool.
    Not(Not<RegisterTy>),
    /// `[dest] = ![dest]`, producing an int.
    UnaryBitNot(UnaryBitNot<RegisterTy>),
    /// `[dest] = [dest] < c`.
    LessThan(LessThan<RegisterTy, Constant>),
    /// `[dest] = [dest] < [src]`.
    LessThanIndirect(LessThanIndirect<RegisterTy, RegisterTy>),
    /// `[dest] = [dest] <= c`.
    LessEqual(LessEqual<RegisterTy, Constant>),
    /// `[dest] = [dest] <= [src]`.
    LessEqualIndirect(LessEqualIndirect<RegisterTy, RegisterTy>),
    /// `[dest] = [dest] > c`.
    GreaterThan(GreaterThan<RegisterTy, Constant>),
    /// `[dest] = [dest] > [src]`.
    GreaterThanIndirect(GreaterThanIndirect<RegisterTy, RegisterTy>),
    /// `[dest] = [dest] >= c`.
    GreaterEqual(GreaterEqual<RegisterTy, Constant>),
    /// `[dest] = [dest] >= [src]`.
    GreaterEqualIndirect(GreaterEqualIndirect<RegisterTy, RegisterTy>),
    /// `[dest] = [dest] == c`.
    Equals(Equals<RegisterTy, Constant>),
    /// `[dest] = [dest] == [src]`.
    EqualsIndirect(EqualsIndirect<RegisterTy, RegisterTy>),
    /// `[dest] = [dest] != c`.
    NotEqual(NotEqual<RegisterTy, Constant>),
    /// `[dest] = [dest] != [src]`.
    NotEqualIndirect(NotEqualIndirect<RegisterTy, RegisterTy>),
    /// `[dest] = [dest] as bool ? c : [dest]`.
    And(And<RegisterTy, Constant>),
    /// `[dest] = [dest] as bool ? [src] : [dest]`.
    AndIndirect(AndIndirect<RegisterTy, RegisterTy>),
    /// `[dest] = [dest] as bool ? [dest] : c`.
    Or(Or<RegisterTy, Constant>),
    /// `[dest] = [dest] as bool ? [dest] : [src]`.
    OrIndirect(OrIndirect<RegisterTy, RegisterTy>),
    /// `[dest] = [dest].to_string() + c.to_string()`.
    Concat(Concat<RegisterTy>),
    /// `[dest] = [dest].to_string() + [src].to_string()`.
    ConcatIndirect(ConcatIndirect<RegisterTy>),
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
    /// `[dest] = `[dest].table[c]`
    Load(Load<RegisterTy>),
    /// `[dest] = `[dest].table[[src]]`
    LoadIndirect(LoadIndirect<RegisterTy>),
    /// `[dest].table[c]` = `[src]`
    Store(Store<RegisterTy>),
    /// `[dest].table[c1]` = `c2`
    StoreConstant(StoreConstant<RegisterTy>),
    /// `[dest].table[c1]` = `va[c2]`
    StoreFromVa(StoreFromVa<RegisterTy>),
    /// `[dest].table[[index]]` = `[src]`
    StoreIndirect(StoreIndirect<RegisterTy>),
    /// `[dest].table[[index]]` = `c`
    StoreConstantIndirect(StoreConstantIndirect<RegisterTy>),
    /// `[dest].table[[index]]` = `va[c]`
    StoreFromVaIndirect(StoreFromVaIndirect<RegisterTy>),
    /// `[dest].table[(start, ..)]` = `va...`
    StoreAllFromVa(StoreAllFromVa<RegisterTy>),
    /// Initialize a register to a constant value.
    Set(Set<RegisterTy>),
    /// Initialize a register from another register.
    SetIndirect(SetIndirect<RegisterTy>),
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
    MapArg(MapArg),
    /// Load the target value into the the next register for the current call
    /// target.
    MapArgIndirect(MapArgIndirect<RegisterTy>),
    /// Copy the first va arg into the next register for the current call target
    MapVa0,
    /// Copy the target constant value into this function's output list.
    SetRet(SetRet),
    /// Copy the target register value into this function's output list.
    SetRetIndirect(SetRetIndirect<RegisterTy>),
    /// Copy the first va arg into this function's output list.
    SetRetVa0,
    /// Copy the first return value from a function into this function's output
    /// list.
    SetRetFromRet0,
    /// Copy all return values from a function into this function's output list
    /// and then return from the function.
    CopyRetFromRetAndRet,
    /// Copy all return values this function's va list and then return from the
    /// function.
    CopyRetFromVaAndRet,
    /// Stop executing this function and return.
    Ret,
    /// Copy the next available return value into the target register.
    MapRet(MapRet<RegisterTy>),
}

#[derive(Debug, Clone, Copy, PartialEq, From, Deref)]
pub struct Concat<RegTy>(BinOpData<Self, RegTy, Constant>);

impl<RegTy> From<(RegTy, Constant)> for Concat<RegTy> {
    fn from(tuple: (RegTy, Constant)) -> Self {
        Self(tuple.into())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, From, Deref)]
pub struct ConcatIndirect<RegTy>(BinOpData<Self, RegTy, RegTy>);

impl<RegTy> From<(RegTy, RegTy)> for ConcatIndirect<RegTy> {
    fn from(tuple: (RegTy, RegTy)) -> Self {
        Self(tuple.into())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, From)]
pub struct UnaryMinus<RegTy> {
    pub reg: RegTy,
}

#[derive(Debug, Clone, Copy, PartialEq, From)]
pub struct UnaryBitNot<RegTy> {
    pub reg: RegTy,
}

#[derive(Debug, Clone, Copy, PartialEq, From)]
pub struct Not<RegTy> {
    pub reg: RegTy,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Length<RegTy> {
    pub reg: RegTy,
}

#[derive(Debug, Clone, Copy, PartialEq, From)]
pub struct Jump {
    pub target: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, From)]
pub struct JumpNot<RegTy> {
    pub cond: RegTy,
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
    pub target: RegTy,
}

#[derive(Debug, Clone, Copy, PartialEq, From)]
pub struct StartCallExtending<RegTy> {
    pub target: RegTy,
}

#[derive(Debug, Clone, Copy, PartialEq, From)]
pub struct MapArg {
    pub value: Constant,
}

#[derive(Debug, Clone, Copy, PartialEq, From)]
pub struct MapArgIndirect<RegTy> {
    pub src: RegTy,
}

#[derive(Debug, Clone, Copy, PartialEq, From)]
pub struct SetRet {
    pub value: Constant,
}

#[derive(Debug, Clone, Copy, PartialEq, From)]
pub struct SetRetIndirect<RegTy> {
    pub src: RegTy,
}

#[derive(Debug, Clone, Copy, PartialEq, From)]
pub struct MapRet<RegTy> {
    pub dest: RegTy,
}

#[derive(Debug, Clone, Copy, PartialEq, From)]
pub struct Raise {
    pub err: OpError,
}

#[derive(Debug, Clone, Copy, PartialEq, From)]
pub struct AllocFunc<RegTy> {
    pub dest: RegTy,
    pub id: FuncId,
}

#[derive(Debug, Clone, Copy, PartialEq, From)]
pub struct AllocTable<RegTy> {
    pub dest: RegTy,
}

#[derive(Debug, Clone, Copy, PartialEq, From)]
pub struct Store<RegTy> {
    pub dest: RegTy,
    pub index: Constant,
    pub src: RegTy,
}

#[derive(Debug, Clone, Copy, PartialEq, From)]
pub struct StoreConstant<RegTy> {
    pub dest: RegTy,
    pub index: Constant,
    pub src: Constant,
}

#[derive(Debug, Clone, Copy, PartialEq, From)]
pub struct StoreFromVa<RegTy> {
    pub dest: RegTy,
    pub index: Constant,
    pub va_index: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, From)]
pub struct StoreIndirect<RegTy> {
    pub dest: RegTy,
    pub index: RegTy,
    pub src: RegTy,
}

#[derive(Debug, Clone, Copy, PartialEq, From)]
pub struct StoreConstantIndirect<RegTy> {
    pub dest: RegTy,
    pub index: RegTy,
    pub src: Constant,
}

#[derive(Debug, Clone, Copy, PartialEq, From)]
pub struct StoreFromVaIndirect<RegTy> {
    pub dest: RegTy,
    pub index: RegTy,
    pub va_index: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, From)]
pub struct StoreAllFromVa<RegTy> {
    pub dest: RegTy,
    pub start_index: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, From)]
pub struct Load<RegTy> {
    pub dest: RegTy,
    pub index: Constant,
}

#[derive(Debug, Clone, Copy, PartialEq, From)]
pub struct LoadIndirect<RegTy> {
    pub dest: RegTy,
    pub index: RegTy,
}

#[derive(Debug, Clone, Copy, PartialEq, From)]
pub struct Set<RegTy> {
    pub dest: RegTy,
    pub source: Constant,
}

#[derive(Debug, Clone, Copy, PartialEq, From)]
pub struct SetIndirect<RegTy> {
    pub dest: RegTy,
    pub source: RegTy,
}

#[derive(Debug, Clone, Copy, PartialEq, From)]
pub struct SetFromVa<RegTy> {
    pub dest: RegTy,
    pub index: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, From)]
pub struct ScopeDescriptor {
    pub size: usize,
}
