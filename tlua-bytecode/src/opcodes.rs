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
};

#[derive(Clone, Copy, PartialEq, From)]
pub enum AnyReg<RegisterTy> {
    Register(MappedRegister<RegisterTy>),
    Immediate(AnonymousRegister),
}

impl<RegisterTy> std::fmt::Debug for AnyReg<RegisterTy>
where
    RegisterTy: std::fmt::Debug,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Register(arg0) => arg0.fmt(f),
            Self::Immediate(arg0) => arg0.fmt(f),
        }
    }
}

#[derive(Clone, Copy, PartialEq, From)]
pub enum Operand<RegisterTy> {
    Nil,
    Bool(bool),
    Float(f64),
    Integer(i64),
    String(ConstantString),
    Register(MappedRegister<RegisterTy>),
    Immediate(AnonymousRegister),
}

impl<RegisterTy> std::fmt::Debug for Operand<RegisterTy>
where
    RegisterTy: std::fmt::Debug,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Nil => write!(f, "nil"),
            Self::Bool(arg0) => arg0.fmt(f),
            Self::Float(arg0) => arg0.fmt(f),
            Self::Integer(arg0) => arg0.fmt(f),
            Self::String(arg0) => arg0.fmt(f),
            Self::Register(arg0) => arg0.fmt(f),
            Self::Immediate(arg0) => arg0.fmt(f),
        }
    }
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
#[derive(Clone, Copy, PartialEq, From)]
pub enum Op<RegisterTy> {
    Nop,
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
    /// Immediately return from the current function with a specific error if
    /// [src] is false.
    RaiseIfNot(RaiseIfNot<RegisterTy>),
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
    Lookup(Lookup<RegisterTy>),
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
    /// [dest] = [src].type == type_id
    CheckType(CheckType<RegisterTy>),
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
    /// Push a new scope as the current local scope.
    PushScope(ScopeDescriptor),
    /// Discard the current scope and restore the most recently pushed scope.
    PopScope,
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

#[derive(Clone, Copy, PartialEq, From, Into)]
pub struct Concat<RegTy> {
    lhs: AnyReg<RegTy>,
    rhs: Operand<RegTy>,
}

impl<Reg> std::fmt::Debug for Concat<Reg>
where
    Reg: std::fmt::Debug,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "concat {:?} {:?}", self.lhs, self.rhs)
    }
}

#[derive(Clone, Copy, PartialEq, From)]
pub struct UnaryMinus<RegTy> {
    pub reg: AnyReg<RegTy>,
}

impl<Reg> std::fmt::Debug for UnaryMinus<Reg>
where
    Reg: std::fmt::Debug,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "-{:?}", self.reg)
    }
}

#[derive(Clone, Copy, PartialEq, From)]
pub struct UnaryBitNot<RegTy> {
    pub reg: AnyReg<RegTy>,
}

impl<Reg> std::fmt::Debug for UnaryBitNot<Reg>
where
    Reg: std::fmt::Debug,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "~{:?}", self.reg)
    }
}

#[derive(Clone, Copy, PartialEq, From)]
pub struct Not<RegTy> {
    pub reg: AnyReg<RegTy>,
}

impl<Reg> std::fmt::Debug for Not<Reg>
where
    Reg: std::fmt::Debug,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "not {:?}", self.reg)
    }
}

#[derive(Clone, Copy, PartialEq)]
pub struct Length<RegTy> {
    pub reg: AnyReg<RegTy>,
}

impl<Reg> std::fmt::Debug for Length<Reg>
where
    Reg: std::fmt::Debug,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "len({:?})", self.reg)
    }
}

#[derive(Clone, Copy, PartialEq, From)]
pub struct Jump {
    pub target: usize,
}

impl std::fmt::Debug for Jump {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "jmp {}", self.target)
    }
}

#[derive(Clone, Copy, PartialEq, From)]
pub struct JumpNot<RegTy> {
    pub cond: AnyReg<RegTy>,
    pub target: usize,
}

impl<Reg> std::fmt::Debug for JumpNot<Reg>
where
    Reg: std::fmt::Debug,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "not {:?} ? jmp {}", self.cond, self.target)
    }
}

#[derive(Clone, Copy, PartialEq, From)]
pub struct JumpNotRet0 {
    pub target: usize,
}

impl std::fmt::Debug for JumpNotRet0 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "not results[0] ? jmp {}", self.target)
    }
}

#[derive(Clone, Copy, PartialEq, From)]
pub struct JumpNotVa0 {
    pub target: usize,
}

impl std::fmt::Debug for JumpNotVa0 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "not va[0] ? {}", self.target)
    }
}

#[derive(Clone, Copy, PartialEq, From)]
pub struct Call<RegTy> {
    pub target: AnyReg<RegTy>,
    pub mapped_args_start: usize,
    pub mapped_args_count: usize,
}

impl<Reg> std::fmt::Debug for Call<Reg>
where
    Reg: std::fmt::Debug,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "call {:?} (imm[{}..{}])",
            self.target,
            self.mapped_args_start,
            self.mapped_args_start + self.mapped_args_count
        )
    }
}

#[derive(Clone, Copy, PartialEq, From)]
pub struct CallCopyRet<RegTy> {
    pub target: AnyReg<RegTy>,
    pub mapped_args_start: usize,
    pub mapped_args_count: usize,
}

impl<Reg> std::fmt::Debug for CallCopyRet<Reg>
where
    Reg: std::fmt::Debug,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "call {:?} (imm[{}..{}], results...)",
            self.target,
            self.mapped_args_start,
            self.mapped_args_start + self.mapped_args_count
        )
    }
}

#[derive(Clone, Copy, PartialEq, From)]
pub struct CallCopyVa<RegTy> {
    pub target: AnyReg<RegTy>,
    pub mapped_args_start: usize,
    pub mapped_args_count: usize,
}

impl<Reg> std::fmt::Debug for CallCopyVa<Reg>
where
    Reg: std::fmt::Debug,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "call {:?} (imm[{}..{}], va...)",
            self.target,
            self.mapped_args_start,
            self.mapped_args_start + self.mapped_args_count
        )
    }
}

#[derive(Clone, Copy, PartialEq, From)]
pub struct SetRet<RegTy> {
    pub src: Operand<RegTy>,
}

impl<Reg> std::fmt::Debug for SetRet<Reg>
where
    Reg: std::fmt::Debug,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "out += {:?}", self.src)
    }
}

#[derive(Clone, Copy, PartialEq, From)]
pub struct Raise {
    pub err: OpError,
}

impl std::fmt::Debug for Raise {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "raise {:?}", self.err)
    }
}

#[derive(Clone, Copy, PartialEq, From)]
pub struct RaiseIfNot<RegTy> {
    pub src: AnyReg<RegTy>,
    pub err: OpError,
}

impl<Reg> std::fmt::Debug for RaiseIfNot<Reg>
where
    Reg: std::fmt::Debug,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "not {:?} ? raise {:?}", self.src, self.err)
    }
}

#[derive(Clone, Copy, PartialEq, From)]
pub struct Alloc<RegTy> {
    pub dest: AnyReg<RegTy>,
    pub type_id: TypeId,
}

impl<Reg> std::fmt::Debug for Alloc<Reg>
where
    Reg: std::fmt::Debug,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "alloc {:?} {:?}", self.dest, self.type_id)
    }
}

#[derive(Clone, Copy, PartialEq, From)]
pub struct CheckType<RegTy> {
    pub dest: AnyReg<RegTy>,
    pub src: AnyReg<RegTy>,
    pub expected_type_id: TypeId,
}

impl<Reg> std::fmt::Debug for CheckType<Reg>
where
    Reg: std::fmt::Debug,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "checktype {:?} = {:?} {:?}",
            self.dest, self.src, self.expected_type_id
        )
    }
}

#[derive(Clone, Copy, PartialEq, From)]
pub struct Store<RegTy> {
    pub dest: AnyReg<RegTy>,
    pub index: Operand<RegTy>,
    pub src: Operand<RegTy>,
}

impl<Reg> std::fmt::Debug for Store<Reg>
where
    Reg: std::fmt::Debug,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}[{:?}] = {:?}", self.dest, self.index, self.src)
    }
}

#[derive(Clone, Copy, PartialEq, From)]
pub struct StoreFromVa<RegTy> {
    pub dest: AnyReg<RegTy>,
    pub index: Operand<RegTy>,
    pub va_index: usize,
}

impl<Reg> std::fmt::Debug for StoreFromVa<Reg>
where
    Reg: std::fmt::Debug,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}[{:?}] = va{}", self.dest, self.index, self.va_index)
    }
}

#[derive(Clone, Copy, PartialEq, From)]
pub struct StoreAllFromVa<RegTy> {
    pub dest: AnyReg<RegTy>,
    pub start_index: usize,
}

impl<Reg> std::fmt::Debug for StoreAllFromVa<Reg>
where
    Reg: std::fmt::Debug,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}[{}..] = va...", self.dest, self.start_index)
    }
}

#[derive(Clone, Copy, PartialEq, From)]
pub struct Lookup<RegTy> {
    pub dest: AnyReg<RegTy>,
    pub index: Operand<RegTy>,
}

impl<Reg> std::fmt::Debug for Lookup<Reg>
where
    Reg: std::fmt::Debug,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}[{:?}]", self.dest, self.index)
    }
}

#[derive(Clone, Copy, PartialEq, From)]
pub struct Set<RegTy> {
    pub dest: AnyReg<RegTy>,
    pub source: Operand<RegTy>,
}

impl<Reg> std::fmt::Debug for Set<Reg>
where
    Reg: std::fmt::Debug,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?} = {:?}", self.dest, self.source)
    }
}

#[derive(Clone, Copy, PartialEq, From)]
pub struct SetFromVa<RegTy> {
    pub dest: AnyReg<RegTy>,
    pub index: usize,
}

impl<Reg> std::fmt::Debug for SetFromVa<Reg>
where
    Reg: std::fmt::Debug,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?} = va[{:?}]", self.dest, self.index)
    }
}

#[derive(Clone, Copy, PartialEq, From)]
pub struct ScopeDescriptor {
    pub size: usize,
}

impl std::fmt::Debug for ScopeDescriptor {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "pushscope {{size = {}}}", self.size)
    }
}

#[derive(Clone, Copy, PartialEq, From)]
pub struct MapRet<RegTy> {
    pub dest: AnyReg<RegTy>,
}

impl<Reg> std::fmt::Debug for MapRet<Reg>
where
    Reg: std::fmt::Debug,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?} = next(results)", self.dest)
    }
}

#[derive(Clone, Copy, PartialEq, From)]
pub struct StoreRet<RegTy> {
    pub dest: AnyReg<RegTy>,
    pub index: Operand<RegTy>,
}

impl<Reg> std::fmt::Debug for StoreRet<Reg>
where
    Reg: std::fmt::Debug,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}[{:?}] = next(results)", self.dest, self.index)
    }
}

#[derive(Clone, Copy, PartialEq, From)]
pub struct StoreAllRet<RegTy> {
    pub dest: AnyReg<RegTy>,
    pub start_index: usize,
}

impl<Reg> std::fmt::Debug for StoreAllRet<Reg>
where
    Reg: std::fmt::Debug,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}[{}..] = results...", self.dest, self.start_index)
    }
}

impl<RegisterTy> std::fmt::Debug for Op<RegisterTy>
where
    RegisterTy: std::fmt::Debug,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Nop => write!(f, "nop"),
            Self::Add(arg0) => arg0.fmt(f),
            Self::Subtract(arg0) => arg0.fmt(f),
            Self::Times(arg0) => arg0.fmt(f),
            Self::Modulo(arg0) => arg0.fmt(f),
            Self::Divide(arg0) => arg0.fmt(f),
            Self::Exponetiation(arg0) => arg0.fmt(f),
            Self::IDiv(arg0) => arg0.fmt(f),
            Self::BitAnd(arg0) => arg0.fmt(f),
            Self::BitOr(arg0) => arg0.fmt(f),
            Self::BitXor(arg0) => arg0.fmt(f),
            Self::ShiftLeft(arg0) => arg0.fmt(f),
            Self::ShiftRight(arg0) => arg0.fmt(f),
            Self::UnaryMinus(arg0) => arg0.fmt(f),
            Self::Not(arg0) => arg0.fmt(f),
            Self::UnaryBitNot(arg0) => arg0.fmt(f),
            Self::LessThan(arg0) => arg0.fmt(f),
            Self::LessEqual(arg0) => arg0.fmt(f),
            Self::GreaterThan(arg0) => arg0.fmt(f),
            Self::GreaterEqual(arg0) => arg0.fmt(f),
            Self::Equals(arg0) => arg0.fmt(f),
            Self::NotEqual(arg0) => arg0.fmt(f),
            Self::And(arg0) => arg0.fmt(f),
            Self::Or(arg0) => arg0.fmt(f),
            Self::Concat(arg0) => arg0.fmt(f),
            Self::Length(arg0) => arg0.fmt(f),
            Self::Raise(arg0) => arg0.fmt(f),
            Self::RaiseIfNot(arg0) => arg0.fmt(f),
            Self::Jump(arg0) => arg0.fmt(f),
            Self::JumpNot(arg0) => arg0.fmt(f),
            Self::JumpNotRet0(arg0) => arg0.fmt(f),
            Self::JumpNotVa0(arg0) => arg0.fmt(f),
            Self::Lookup(arg0) => arg0.fmt(f),
            Self::Store(arg0) => arg0.fmt(f),
            Self::StoreFromVa(arg0) => arg0.fmt(f),
            Self::StoreAllFromVa(arg0) => arg0.fmt(f),
            Self::Set(arg0) => arg0.fmt(f),
            Self::SetFromVa(arg0) => arg0.fmt(f),
            Self::Alloc(arg0) => arg0.fmt(f),
            Self::CheckType(arg0) => arg0.fmt(f),
            Self::SetRet(arg0) => arg0.fmt(f),
            Self::SetRetVa0 => write!(f, "out += va[0]"),
            Self::SetRetFromRet0 => write!(f, "out += results[0]"),
            Self::CopyRetFromVaAndRet => write!(f, "out += va...; ret"),
            Self::Ret => write!(f, "ret"),
            Self::PushScope(arg0) => arg0.fmt(f),
            Self::PopScope => write!(f, "popscope"),
            Self::Call(arg0) => arg0.fmt(f),
            Self::CallCopyRet(arg0) => arg0.fmt(f),
            Self::CallCopyVa(arg0) => arg0.fmt(f),
            Self::CopyRetFromRetAndRet => write!(f, "out += results...; ret"),
            Self::MapRet(arg0) => arg0.fmt(f),
            Self::StoreRet(arg0) => arg0.fmt(f),
            Self::StoreAllRet(arg0) => arg0.fmt(f),
        }
    }
}
