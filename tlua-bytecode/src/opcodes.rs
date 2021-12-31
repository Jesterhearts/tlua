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

pub type Instruction = Op<Register>;

#[derive(Debug, Clone, Copy, PartialEq, From)]
pub enum Op<RegisterTy> {
    /// `[this] += c`, preserving types.
    Add(Add<RegisterTy, Constant>),
    /// `[this] += [other]`
    AddIndirect(AddIndirect<RegisterTy, RegisterTy>),
    /// `[this] -= c`, preserving types.
    Subtract(Subtract<RegisterTy, Constant>),
    /// `[this] -= [other]`, preserving types.
    SubtractIndirect(SubtractIndirect<RegisterTy, RegisterTy>),
    /// `[this] *= c`, preserving types.
    Times(Times<RegisterTy, Constant>),
    /// `[this] *= [other]`, preserving types.
    TimesIndirect(TimesIndirect<RegisterTy, RegisterTy>),
    /// `[this] %= c`, preserving types.
    Modulo(Modulo<RegisterTy, Constant>),
    /// `[this] %= [other]`, preserving types.
    ModuloIndirect(ModuloIndirect<RegisterTy, RegisterTy>),
    // Floating point operations
    /// `[this] = [this] / c`, producing a float.
    Divide(Divide<RegisterTy, Constant>),
    /// `[this] = [this] / [other]`, producing a float.
    DivideIndirect(DivideIndirect<RegisterTy, RegisterTy>),
    /// `[this] = [this].exp(c)`, producing a float.
    Exponetiation(Exponetiation<RegisterTy, Constant>),
    /// `[this] = [this].exp([other])`, producing a float.
    ExponetiationIndirect(ExponetiationIndirect<RegisterTy, RegisterTy>),
    /// `[this] = floor([this] / c)`, type preserving.
    IDiv(IDiv<RegisterTy, Constant>),
    /// `[this] = floor([this] / [other])`, type preserving.
    IDivIndirect(IDivIndirect<RegisterTy, RegisterTy>),
    /// `[this] = [this] & c`, producing an int.
    BitAnd(BitAnd<RegisterTy, Constant>),
    /// `[this] = [this] & [other]`, producing an int.
    BitAndIndirect(BitAndIndirect<RegisterTy, RegisterTy>),
    /// `[this] = [this] | c`, producing an int.
    BitOr(BitOr<RegisterTy, Constant>),
    /// `[this] = [this] | [other]`, producing an int.
    BitOrIndirect(BitOrIndirect<RegisterTy, RegisterTy>),
    /// `[this] = [this] ^ c`, producing an int.
    BitXor(BitXor<RegisterTy, Constant>),
    /// `[this] = [this] ^ [other]`, producing an int.
    BitXorIndirect(BitXorIndirect<RegisterTy, RegisterTy>),
    /// `[this] = [this] << c`, producing an int.
    ShiftLeft(ShiftLeft<RegisterTy, Constant>),
    /// `[this] = [this] << [other]`, producing an int.
    ShiftLeftIndirect(ShiftLeftIndirect<RegisterTy, RegisterTy>),
    /// `[this] = [this] >> c`, producing an int.
    ShiftRight(ShiftRight<RegisterTy, Constant>),
    /// `[this] = [this] >> [other]`, producing an int.
    ShiftRightIndirect(ShiftRightIndirect<RegisterTy, RegisterTy>),
    /// `[this] = -[this]`, type preserving.
    UnaryMinus(UnaryMinus<RegisterTy>),
    /// `[this] = !([this] as bool)`, producing a bool.
    Not(Not<RegisterTy>),
    /// `[this] = ![this]`, producing an int.
    UnaryBitNot(UnaryBitNot<RegisterTy>),
    /// `[this] = [this] < c`.
    LessThan(LessThan<RegisterTy, Constant>),
    /// `[this] = [this] < [other]`.
    LessThanIndirect(LessThanIndirect<RegisterTy, RegisterTy>),
    /// `[this] = [this] <= c`.
    LessEqual(LessEqual<RegisterTy, Constant>),
    /// `[this] = [this] <= [other]`.
    LessEqualIndirect(LessEqualIndirect<RegisterTy, RegisterTy>),
    /// `[this] = [this] > c`.
    GreaterThan(GreaterThan<RegisterTy, Constant>),
    /// `[this] = [this] > [other]`.
    GreaterThanIndirect(GreaterThanIndirect<RegisterTy, RegisterTy>),
    /// `[this] = [this] >= c`.
    GreaterEqual(GreaterEqual<RegisterTy, Constant>),
    /// `[this] = [this] >= [other]`.
    GreaterEqualIndirect(GreaterEqualIndirect<RegisterTy, RegisterTy>),
    /// `[this] = [this] == c`.
    Equals(Equals<RegisterTy, Constant>),
    /// `[this] = [this] == [other]`.
    EqualsIndirect(EqualsIndirect<RegisterTy, RegisterTy>),
    /// `[this] = [this] != c`.
    NotEqual(NotEqual<RegisterTy, Constant>),
    /// `[this] = [this] != [other]`.
    NotEqualIndirect(NotEqualIndirect<RegisterTy, RegisterTy>),
    /// `[this] = [this] as bool ? c : [this]`.
    And(And<RegisterTy, Constant>),
    /// `[this] = [this] as bool ? [other] : [this]`.
    AndIndirect(AndIndirect<RegisterTy, RegisterTy>),
    /// `[this] = [this] as bool ? [this] : c`.
    Or(Or<RegisterTy, Constant>),
    /// `[this] = [this] as bool ? [this] : [other]`.
    OrIndirect(OrIndirect<RegisterTy, RegisterTy>),
    /// `[this] = [this].to_string() + c.to_string()`.
    Concat(Concat<RegisterTy>),
    /// `[this] = [this].to_string() + [other].to_string()`.
    ConcatIndirect(ConcatIndirect<RegisterTy>),
    /// `[this] = [this].len()`.
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
    /// `[this] = `[this].table[c]`
    Load(Load<RegisterTy>),
    /// `[this] = `[this].table[[other]]`
    LoadIndirect(LoadIndirect<RegisterTy>),
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
