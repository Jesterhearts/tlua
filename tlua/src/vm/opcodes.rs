use derive_more::{
    Deref,
    From,
};

use crate::vm::{
    binop::*,
    Constant,
    FuncId,
    OpError,
    Register,
};

pub type Instruction = Op<Register>;

#[derive(Debug, Clone, Copy, PartialEq, From)]
pub enum Op<RegisterTy> {
    // Floating point & integer operations.
    Add(Add<RegisterTy, Constant>),
    AddIndirect(AddIndirect<RegisterTy, RegisterTy>),
    Subtract(Subtract<RegisterTy, Constant>),
    SubtractIndirect(SubtractIndirect<RegisterTy, RegisterTy>),
    Times(Times<RegisterTy, Constant>),
    TimesIndirect(TimesIndirect<RegisterTy, RegisterTy>),
    Modulo(Modulo<RegisterTy, Constant>),
    ModuloIndirect(ModuloIndirect<RegisterTy, RegisterTy>),
    // Floating point operations
    Divide(Divide<RegisterTy, Constant>),
    DivideIndirect(DivideIndirect<RegisterTy, RegisterTy>),
    Exponetiation(Exponetiation<RegisterTy, Constant>),
    ExponetiationIndirect(ExponetiationIndirect<RegisterTy, RegisterTy>),
    // Integer operations
    IDiv(IDiv<RegisterTy, Constant>),
    IDivIndirect(IDivIndirect<RegisterTy, RegisterTy>),
    BitAnd(BitAnd<RegisterTy, Constant>),
    BitAndIndirect(BitAndIndirect<RegisterTy, RegisterTy>),
    BitOr(BitOr<RegisterTy, Constant>),
    BitOrIndirect(BitOrIndirect<RegisterTy, RegisterTy>),
    BitXor(BitXor<RegisterTy, Constant>),
    BitXorIndirect(BitXorIndirect<RegisterTy, RegisterTy>),
    ShiftLeft(ShiftLeft<RegisterTy, Constant>),
    ShiftLeftIndirect(ShiftLeftIndirect<RegisterTy, RegisterTy>),
    ShiftRight(ShiftRight<RegisterTy, Constant>),
    ShiftRightIndirect(ShiftRightIndirect<RegisterTy, RegisterTy>),
    // Unary operations
    UnaryMinus(UnaryMinus<RegisterTy>),
    Not(Not<RegisterTy>),
    UnaryBitNot(UnaryBitNot<RegisterTy>),
    // Comparison operations
    LessThan(LessThan<RegisterTy, Constant>),
    LessThanIndirect(LessThanIndirect<RegisterTy, RegisterTy>),
    LessEqual(LessEqual<RegisterTy, Constant>),
    LessEqualIndirect(LessEqualIndirect<RegisterTy, RegisterTy>),
    GreaterThan(GreaterThan<RegisterTy, Constant>),
    GreaterThanIndirect(GreaterThanIndirect<RegisterTy, RegisterTy>),
    GreaterEqual(GreaterEqual<RegisterTy, Constant>),
    GreaterEqualIndirect(GreaterEqualIndirect<RegisterTy, RegisterTy>),
    Equals(Equals<RegisterTy, Constant>),
    EqualsIndirect(EqualsIndirect<RegisterTy, RegisterTy>),
    NotEqual(NotEqual<RegisterTy, Constant>),
    NotEqualIndirect(NotEqualIndirect<RegisterTy, RegisterTy>),
    // Boolean operations
    And(And<RegisterTy, Constant>),
    AndIndirect(AndIndirect<RegisterTy, RegisterTy>),
    Or(Or<RegisterTy, Constant>),
    OrIndirect(OrIndirect<RegisterTy, RegisterTy>),
    // String & array operations
    Concat(Concat<RegisterTy>),
    ConcatIndirect(ConcatIndirect<RegisterTy>),
    Length(Length<RegisterTy>),
    // Control flow
    Raise(Raise),
    // Unconditionally jump to the targt instruction
    Jump(Jump),
    // Jump to a specific instruction if the value in the register evaluates to false.
    JumpNot(JumpNot<RegisterTy>),
    // Jump to a specific instruction if the first return value evaluates to false.
    JumpNotRet0(JumpNotRet0),
    // Jump to a specific instruction if the first variadic argument
    JumpNotVa0(JumpNotVa0),
    // Register operations
    Set(Set<RegisterTy>),
    SetIndirect(SetIndirect<RegisterTy>),
    SetFromVa(SetFromVa<RegisterTy>),
    /// Allocate a new function
    AllocFunc(AllocFunc<RegisterTy>),
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
    /// Copy the target value into this function's output list.
    SetRet(SetRet),
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
    /// Copy the list of values from
    /// Stop executing this function and return.
    Ret,
    /// Copy the next available return value into the target register.
    MapRet(MapRet<RegisterTy>),
}

#[derive(Debug, Clone, Copy, PartialEq, From, Deref)]
pub struct Concat<RegTy>(BinOp<Self, RegTy, Constant>);

impl<RegTy> From<(RegTy, Constant)> for Concat<RegTy> {
    fn from(tuple: (RegTy, Constant)) -> Self {
        Self(tuple.into())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, From, Deref)]
pub struct ConcatIndirect<RegTy>(BinOp<Self, RegTy, RegTy>);

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
