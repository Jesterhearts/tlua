use derive_more::From;
use tlua_parser::expressions::strings::ConstantString;

pub use crate::binop::*;
use crate::{
    register::{
        ImmediateRegister,
        MappedRegister,
        Register,
    },
    NumLike,
    Number,
    OpError,
    Truthy,
    TypeId,
};

#[derive(Clone, Copy, PartialEq, From)]
pub enum Constant {
    Nil,
    Bool(bool),
    Float(f64),
    Integer(i64),
    String(ConstantString),
}

impl Truthy for Constant {
    fn as_bool(&self) -> bool {
        match self {
            Constant::Nil => false,
            Constant::Bool(b) => *b,
            _ => true,
        }
    }
}

impl NumLike for Constant {
    fn as_float(&self) -> Option<f64> {
        match self {
            Constant::Float(f) => Some(*f),
            Constant::Integer(i) => Some(*i as f64),
            _ => None,
        }
    }

    fn as_int(&self) -> Option<i64> {
        match self {
            Constant::Integer(i) => Some(*i),
            _ => None,
        }
    }
}

impl Constant {
    pub fn short_type_name(&self) -> &'static str {
        match self {
            Constant::Nil => "nil",
            Constant::Bool(_) => "bool",
            Constant::Float(_) | Constant::Integer(_) => "number",
            Constant::String(_) => "string",
        }
    }
}

impl std::fmt::Debug for Constant {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Nil => write!(f, "nil"),
            Self::Bool(arg0) => arg0.fmt(f),
            Self::Float(arg0) => arg0.fmt(f),
            Self::Integer(arg0) => arg0.fmt(f),
            Self::String(arg0) => arg0.fmt(f),
        }
    }
}

impl From<Number> for Constant {
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
    Add(Add),
    /// `[dest] -= [src]`, preserving types.
    Subtract(Subtract),
    /// `[dest] *= [src]`, preserving types.
    Times(Times),
    /// `[dest] %= [src]`, preserving types.
    Modulo(Modulo),
    /// `[dest] = [dest] / [src]`, producing a float.
    Divide(Divide),
    /// `[dest] = [dest].exp([src])`, producing a float.
    Exponetiation(Exponetiation),
    /// `[dest] = floor([dest] / [src])`, type preserving.
    IDiv(IDiv),
    /// `[dest] = [dest] & [src]`, producing an int.
    BitAnd(BitAnd),
    /// `[dest] = [dest] | [src]`, producing an int.
    BitOr(BitOr),
    /// `[dest] = [dest] ^ [src]`, producing an int.
    BitXor(BitXor),
    /// `[dest] = [dest] << [src]`, producing an int.
    ShiftLeft(ShiftLeft),
    /// `[dest] = [dest] >> [src]`, producing an int.
    ShiftRight(ShiftRight),
    /// `[dest] = -[dest]`, type preserving.
    UnaryMinus(UnaryMinus),
    /// `[dest] = ![dest]`, producing an int.
    UnaryBitNot(UnaryBitNot),
    /// `[dest] = !([dest] as bool)`, producing a bool.
    Not(Not),
    /// `[dest] = [dest] < [src]`.
    LessThan(LessThan),
    /// `[dest] = [dest] <= [src]`.
    LessEqual(LessEqual),
    /// `[dest] = [dest] > [src]`.
    GreaterThan(GreaterThan),
    /// `[dest] = [dest] >= [src]`.
    GreaterEqual(GreaterEqual),
    /// `[dest] = [dest] == [src]`.
    Equals(Equals),
    /// `[dest] = [dest] != [src]`.
    NotEqual(NotEqual),
    /// `[dest] = [dest] as bool ? [src] : [dest]`.
    And(And),
    /// `[dest] = [dest] as bool ? [dest] : [src]`.
    Or(Or),
    /// `[dest] = [dest].to_string() + [src].to_string()`.
    Concat(Concat),
    /// `[dest] = [dest].len()`.
    Length(Length),
    /// Immediately return from the current function with a specific error.
    Raise(Raise),
    /// Immediately return from the current function with a specific error if
    /// [src] is false.
    RaiseIfNot(RaiseIfNot),
    /// Unconditionally jump to the targt instruction
    Jump(Jump),
    /// Jump to a specific instruction if the value in the register evaluates to
    /// false.
    JumpNot(JumpNot),
    /// Jump to a specific instruction if the value in the register is exactly
    /// Nil
    JumpNil(JumpNil),
    /// `[dest] = `[dest].table[[src]]`
    Lookup(Lookup),
    /// `[dest].table[[index]]` = `[src]`
    SetProperty(SetProperty),
    /// `[dest].table[(start, ..)]` = `va...`
    SetAllPropertiesFromVa(SetAllPropertiesFromVa),
    /// Initialize a register from a value.
    LoadConstant(LoadConstant),
    /// Initialize a register from a mapped register.
    LoadRegister(LoadRegister<RegisterTy>),
    /// Initialize a register from a register.
    DuplicateRegister(DuplicateRegister),
    /// Initialize a register from a variadic argument.
    LoadVa(LoadVa),
    /// Initialize a mapped register register from a register.
    Store(Store<RegisterTy>),
    /// Allocate a type
    Alloc(Alloc),
    /// [dest] = [src].type == type_id
    CheckType(CheckType),
    /// Copy the target register value into this function's output list.
    SetRet(SetRet),
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
    /// immediate register as that function's arguments.
    Call(Call),
    /// Performs the same operations as `Call` but maps the results of the most
    /// recent call into the target's arguments.
    CallCopyRet(CallCopyRet),
    /// Performs the same operations as `Call`, but maps the current list of
    /// variadic arguments into the target's arguments.
    CallCopyVa(CallCopyVa),
    /// Copy all return values from a function into this function's output list
    /// and then return from the function.
    CopyRetFromRetAndRet,
    /// Copy the next available return value into the target register.
    ConsumeRetRange(ConsumeRetRange),
    /// Copy all the available return values into a table.
    SetAllPropertiesFromRet(SetAllPropertiesFromRet),
}

#[derive(Clone, Copy, PartialEq, From)]
pub struct UnaryMinus {
    pub dst: ImmediateRegister,
    pub src: ImmediateRegister,
}

impl std::fmt::Debug for UnaryMinus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?} = -{:?}", self.dst, self.src)
    }
}

#[derive(Clone, Copy, PartialEq, From)]
pub struct UnaryBitNot {
    pub dst: ImmediateRegister,
    pub src: ImmediateRegister,
}

impl std::fmt::Debug for UnaryBitNot {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?} = ~{:?}", self.dst, self.src)
    }
}

#[derive(Clone, Copy, PartialEq, From)]
pub struct Not {
    pub dst: ImmediateRegister,
    pub src: ImmediateRegister,
}

impl std::fmt::Debug for Not {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?} = !{:?}", self.dst, self.src)
    }
}

#[derive(Clone, Copy, PartialEq, From)]
pub struct Length {
    pub dst: ImmediateRegister,
    pub src: ImmediateRegister,
}

impl std::fmt::Debug for Length {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?} = len({:?})", self.dst, self.src)
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
pub struct JumpNot {
    pub cond: ImmediateRegister,
    pub target: usize,
}

impl std::fmt::Debug for JumpNot {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "not {:?} ? jmp {}", self.cond, self.target)
    }
}

#[derive(Clone, Copy, PartialEq, From)]
pub struct JumpNil {
    pub cond: ImmediateRegister,
    pub target: usize,
}

impl std::fmt::Debug for JumpNil {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "isnil {:?} ? jmp {}", self.cond, self.target)
    }
}

#[derive(Clone, Copy, PartialEq, From)]
pub struct Call {
    pub target: ImmediateRegister,
    pub mapped_args_start: usize,
    pub mapped_args_count: usize,
}

impl std::fmt::Debug for Call {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "call {:?} ({:?}..{:?})",
            self.target,
            ImmediateRegister::from(self.mapped_args_start),
            ImmediateRegister::from(self.mapped_args_start + self.mapped_args_count)
        )
    }
}

#[derive(Clone, Copy, PartialEq, From)]
pub struct CallCopyRet {
    pub target: ImmediateRegister,
    pub mapped_args_start: usize,
    pub mapped_args_count: usize,
}

impl std::fmt::Debug for CallCopyRet {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "call {:?} ({:?}..{:?}, results...)",
            self.target,
            ImmediateRegister::from(self.mapped_args_start),
            ImmediateRegister::from(self.mapped_args_start + self.mapped_args_count)
        )
    }
}

#[derive(Clone, Copy, PartialEq, From)]
pub struct CallCopyVa {
    pub target: ImmediateRegister,
    pub mapped_args_start: usize,
    pub mapped_args_count: usize,
}

impl std::fmt::Debug for CallCopyVa {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "call {:?} ({:?}..{:?}, results...)",
            self.target,
            ImmediateRegister::from(self.mapped_args_start),
            ImmediateRegister::from(self.mapped_args_start + self.mapped_args_count)
        )
    }
}

#[derive(Clone, Copy, PartialEq, From)]
pub struct SetRet {
    pub src: ImmediateRegister,
}

impl std::fmt::Debug for SetRet {
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
pub struct RaiseIfNot {
    pub src: ImmediateRegister,
    pub err: OpError,
}

impl std::fmt::Debug for RaiseIfNot {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "not {:?} ? raise {:?}", self.src, self.err)
    }
}

#[derive(Clone, Copy, PartialEq, From)]
pub struct Alloc {
    pub dst: ImmediateRegister,
    pub type_id: TypeId,
}

impl std::fmt::Debug for Alloc {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "alloc {:?} {:?}", self.dst, self.type_id)
    }
}

#[derive(Clone, Copy, PartialEq, From)]
pub struct CheckType {
    pub dst: ImmediateRegister,
    pub src: ImmediateRegister,
    pub expected_type_id: TypeId,
}

impl std::fmt::Debug for CheckType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "checktype {:?} = {:?} {:?}",
            self.dst, self.dst, self.expected_type_id
        )
    }
}

#[derive(Clone, Copy, PartialEq, From)]
pub struct SetProperty {
    pub dst: ImmediateRegister,
    pub idx: ImmediateRegister,
    pub src: ImmediateRegister,
}

impl std::fmt::Debug for SetProperty {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}[{:?}] = {:?}", self.dst, self.idx, self.src)
    }
}

#[derive(Clone, Copy, PartialEq, From)]
pub struct SetAllPropertiesFromVa {
    pub dst: ImmediateRegister,
    pub start_idx: usize,
}

impl std::fmt::Debug for SetAllPropertiesFromVa {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}[{}..] = va...", self.dst, self.start_idx)
    }
}

#[derive(Clone, Copy, PartialEq, From)]
pub struct Lookup {
    pub dst: ImmediateRegister,
    pub src: ImmediateRegister,
    pub idx: ImmediateRegister,
}

impl std::fmt::Debug for Lookup {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?} = {:?}[{:?}]", self.dst, self.src, self.idx)
    }
}

#[derive(Clone, Copy, PartialEq, From)]
pub struct LoadConstant {
    pub dst: ImmediateRegister,
    pub src: Constant,
}

impl std::fmt::Debug for LoadConstant {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?} = {:?}", self.dst, self.src)
    }
}

#[derive(Clone, Copy, PartialEq, From)]
pub struct LoadRegister<RegTy> {
    pub dst: ImmediateRegister,
    pub src: MappedRegister<RegTy>,
}

impl<Reg> std::fmt::Debug for LoadRegister<Reg>
where
    Reg: std::fmt::Debug,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?} = {:?}", self.dst, self.src)
    }
}

#[derive(Clone, Copy, PartialEq, From)]
pub struct DuplicateRegister {
    pub dst: ImmediateRegister,
    pub src: ImmediateRegister,
}

impl std::fmt::Debug for DuplicateRegister {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?} = {:?}", self.dst, self.src)
    }
}

#[derive(Clone, Copy, PartialEq, From)]
pub struct LoadVa {
    pub dst_start: usize,
    pub va_start: usize,
    pub count: usize,
}

impl std::fmt::Debug for LoadVa {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{:?}..{:?} = va[{}..]",
            ImmediateRegister::from(self.dst_start),
            ImmediateRegister::from(self.dst_start + self.va_start),
            self.count
        )
    }
}

#[derive(Clone, Copy, PartialEq, From)]
pub struct Store<RegTy> {
    pub dst: MappedRegister<RegTy>,
    pub src: ImmediateRegister,
}

impl<Reg> std::fmt::Debug for Store<Reg>
where
    Reg: std::fmt::Debug,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?} = {:?}", self.dst, self.src)
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
pub struct ConsumeRetRange {
    pub dst_start: usize,
    pub count: usize,
}

impl std::fmt::Debug for ConsumeRetRange {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{:?}..{:?} = results[..{}]",
            ImmediateRegister::from(self.dst_start),
            ImmediateRegister::from(self.dst_start + self.count),
            self.count,
        )
    }
}

#[derive(Clone, Copy, PartialEq, From)]
pub struct SetAllPropertiesFromRet {
    pub dst: ImmediateRegister,
    pub start_idx: usize,
}

impl std::fmt::Debug for SetAllPropertiesFromRet {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}[{}..] = results...", self.dst, self.start_idx)
    }
}

impl<RegisterTy> std::fmt::Debug for Op<RegisterTy>
where
    RegisterTy: std::fmt::Debug,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Op::Nop => write!(f, "nop"),
            Op::Add(op) => op.fmt(f),
            Op::Subtract(op) => op.fmt(f),
            Op::Times(op) => op.fmt(f),
            Op::Modulo(op) => op.fmt(f),
            Op::Divide(op) => op.fmt(f),
            Op::Exponetiation(op) => op.fmt(f),
            Op::IDiv(op) => op.fmt(f),
            Op::BitAnd(op) => op.fmt(f),
            Op::BitOr(op) => op.fmt(f),
            Op::BitXor(op) => op.fmt(f),
            Op::ShiftLeft(op) => op.fmt(f),
            Op::ShiftRight(op) => op.fmt(f),
            Op::UnaryMinus(op) => op.fmt(f),
            Op::UnaryBitNot(op) => op.fmt(f),
            Op::Not(op) => op.fmt(f),
            Op::LessThan(op) => op.fmt(f),
            Op::LessEqual(op) => op.fmt(f),
            Op::GreaterThan(op) => op.fmt(f),
            Op::GreaterEqual(op) => op.fmt(f),
            Op::Equals(op) => op.fmt(f),
            Op::NotEqual(op) => op.fmt(f),
            Op::And(op) => op.fmt(f),
            Op::Or(op) => op.fmt(f),
            Op::Concat(op) => op.fmt(f),
            Op::Length(op) => op.fmt(f),
            Op::Raise(op) => op.fmt(f),
            Op::RaiseIfNot(op) => op.fmt(f),
            Op::Jump(op) => op.fmt(f),
            Op::JumpNot(op) => op.fmt(f),
            Op::JumpNil(op) => op.fmt(f),
            Op::Lookup(op) => op.fmt(f),
            Op::SetProperty(op) => op.fmt(f),
            Op::SetAllPropertiesFromVa(op) => op.fmt(f),
            Op::LoadConstant(op) => op.fmt(f),
            Op::LoadRegister(op) => op.fmt(f),
            Op::DuplicateRegister(op) => op.fmt(f),
            Op::LoadVa(op) => op.fmt(f),
            Op::Store(op) => op.fmt(f),
            Op::Alloc(op) => op.fmt(f),
            Op::CheckType(op) => op.fmt(f),
            Op::SetRet(op) => op.fmt(f),
            Op::CopyRetFromVaAndRet => {
                write!(f, "ret out += va...")
            }
            Op::Ret => write!(f, "ret"),
            Op::PushScope(op) => op.fmt(f),
            Op::PopScope => write!(f, "popscope"),
            Op::Call(op) => op.fmt(f),
            Op::CallCopyRet(op) => op.fmt(f),
            Op::CallCopyVa(op) => op.fmt(f),
            Op::CopyRetFromRetAndRet => write!(f, "ret out += results..."),
            Op::ConsumeRetRange(op) => op.fmt(f),
            Op::SetAllPropertiesFromRet(op) => op.fmt(f),
        }
    }
}
