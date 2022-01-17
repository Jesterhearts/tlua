use std::{
    num::NonZeroUsize,
    ops::{
        Deref,
        DerefMut,
    },
};

use derive_more::From;
use tlua_bytecode::{
    opcodes,
    AnonymousRegister,
    ByteCodeError,
    OpError,
    TypeMeta,
};
use tlua_parser::ast::{
    block::Block,
    identifiers::Ident,
};

use crate::{
    constant::Constant,
    Chunk,
    CompileError,
    CompileExpression,
    CompileStatement,
    NodeOutput,
    TypeIds,
};

mod scope;
pub(super) mod unasm;

use self::{
    scope::*,
    unasm::*,
};

#[derive(Debug, Clone, Copy)]
pub(crate) enum VariableTarget {
    Ident(Ident),
    // TODO(tables)
    Register(UnasmRegister),
}

#[derive(Debug, Clone, Copy)]
pub(crate) enum LocalVariableTarget {
    Mutable(Ident),
    Constant(Ident),
    Closable(Ident),
}

#[derive(Debug)]
pub(crate) enum HasVaArgs {
    None,
    Some,
}

#[derive(Debug, Default)]
pub(crate) struct Compiler {
    scope: Scope,

    functions: Vec<UnasmFunction>,
}

impl Compiler {
    pub(crate) fn compile_ast(mut self, ast: Block) -> Result<Chunk, CompileError> {
        let mut main = self.new_context();

        let _ = ast.compile(&mut main)?;

        let main = main.complete();

        Ok(self.into_chunk(main))
    }

    pub(super) fn new_context(&mut self) -> MainCompilerContext {
        MainCompilerContext {
            context: CompilerContext {
                functions: &mut self.functions,

                scope: self.scope.new_context(GLOBAL_SCOPE.into()),

                has_va_args: HasVaArgs::None,

                function: UnasmFunction::default(),
            },
        }
    }

    fn into_chunk(self, main: UnasmFunction) -> Chunk {
        let Self {
            scope: Scope { globals, .. },
            functions,
        } = self;

        Chunk {
            globals_map: globals
                .into_iter()
                .map(|(global, reg)| {
                    debug_assert_eq!(reg.source_scope, GLOBAL_SCOPE);
                    (global, reg.offset.into())
                })
                .collect(),
            functions: functions
                .into_iter()
                .map(|func| func.into_function())
                .collect(),
            main: main.into_function(),
        }
    }
}

trait RegisterMappable<RegisterTy>: Sized
where
    RegisterTy: InitRegister<RegisterTy> + Copy,
    UnasmRegister: From<RegisterTy>,
{
    fn map(self, compiler: &mut CompilerContext) -> Result<RegisterTy, CompileError>;
}

impl RegisterMappable<MappedLocalRegister> for LocalVariableTarget {
    fn map(self, compiler: &mut CompilerContext) -> Result<MappedLocalRegister, CompileError> {
        match self {
            LocalVariableTarget::Mutable(ident) => {
                Ok(compiler.new_local(ident)?.no_init_needed().into())
            }
            LocalVariableTarget::Constant(ident) => {
                Ok(compiler.scope.new_constant(ident)?.no_init_needed().into())
            }
            LocalVariableTarget::Closable(_) => todo!(),
        }
    }
}

impl RegisterMappable<UnasmRegister> for VariableTarget {
    fn map(self, compiler: &mut CompilerContext) -> Result<UnasmRegister, CompileError> {
        match self {
            VariableTarget::Ident(ident) => Ok(compiler.scope.get_in_scope(ident)?.into()),
            VariableTarget::Register(reg) => Ok(reg),
        }
    }
}

pub(crate) trait InitRegister<RegisterTy>: Sized
where
    RegisterTy: Copy,
    UnasmRegister: From<RegisterTy>,
{
    /// Indicate that the register should always init to nil, and needs no
    /// special handling.
    fn no_init_needed(self) -> RegisterTy;

    /// Indicate that the register should be initialized from a return value.
    fn init_from_ret(self, compiler: &mut CompilerContext) -> RegisterTy {
        let register = self.no_init_needed();
        compiler.emit(opcodes::MapRet::from(UnasmRegister::from(register)));
        register
    }

    /// Indicate that the register should be initialized to a constant. If the
    /// constant is always nil, please use init_from_nil.
    fn init_from_const(self, compiler: &mut CompilerContext, value: Constant) -> RegisterTy {
        let register = self.no_init_needed();
        compiler.emit(opcodes::Set::from((
            UnasmRegister::from(register),
            UnasmOperand::from(value),
        )));
        register
    }

    /// Indicate the the register should be initialized by allocating a
    /// function.
    fn init_alloc_fn(self, compiler: &mut CompilerContext, value: TypeMeta) -> RegisterTy {
        let register = self.no_init_needed();
        compiler.emit(opcodes::Alloc::from((
            UnasmRegister::from(register),
            TypeIds::FUNCTION,
            value,
        )));
        register
    }

    /// Indicate the the register shoudl be initialized by allocating a table
    fn init_alloc_table(self, compiler: &mut CompilerContext) -> RegisterTy {
        let register = self.no_init_needed();
        compiler.emit(opcodes::Alloc::from((
            UnasmRegister::from(register),
            TypeIds::TABLE,
            TypeMeta::from(None),
        )));
        register
    }

    /// Indicate that the register should be initialized from another register.
    fn init_from_reg(self, compiler: &mut CompilerContext, other: UnasmRegister) -> RegisterTy {
        let register = self.no_init_needed();
        compiler.emit(opcodes::Set::from((
            UnasmRegister::from(register),
            UnasmOperand::from(other),
        )));
        register
    }

    fn init_from_va(self, compiler: &mut CompilerContext, index: usize) -> RegisterTy {
        let register = self.no_init_needed();
        compiler.emit(opcodes::SetFromVa::from((
            UnasmRegister::from(register),
            index,
        )));
        register
    }
}

impl InitRegister<UnasmRegister> for UnasmRegister {
    fn no_init_needed(self) -> UnasmRegister {
        self
    }
}

impl InitRegister<MappedLocalRegister> for MappedLocalRegister {
    fn no_init_needed(self) -> Self {
        self
    }
}

#[derive(Debug, From)]
#[must_use]
pub(crate) struct UninitRegister<RegisterTy> {
    register: RegisterTy,
}

impl<RegisterTy> InitRegister<RegisterTy> for UninitRegister<RegisterTy>
where
    RegisterTy: Copy,
    UnasmRegister: From<RegisterTy>,
{
    fn no_init_needed(self) -> RegisterTy {
        self.register
    }
}

#[derive(Debug)]
pub(crate) struct MainCompilerContext<'chunk> {
    context: CompilerContext<'chunk>,
}

impl<'chunk> Deref for MainCompilerContext<'chunk> {
    type Target = CompilerContext<'chunk>;

    fn deref(&self) -> &Self::Target {
        &self.context
    }
}

impl DerefMut for MainCompilerContext<'_> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.context
    }
}

impl MainCompilerContext<'_> {
    pub(crate) fn complete(self) -> UnasmFunction {
        let Self {
            context:
                CompilerContext {
                    functions: _,
                    scope,
                    has_va_args: _,
                    mut function,
                },
        } = self;

        function.anon_registers = scope.total_anons;
        function.local_registers = scope.total_locals;
        function
    }
}

/// Instruction sequence creation. This is implemented here rather than exposing
/// a global write function since the compiler needs to make sure that e.g. the
/// stack is reset prior to a return or raise and it is tricky to get that write
/// in the AST-walking portion of the code.
/// e.g. It's a lot easier to make sure the
/// stack is cleared before every raise instruction if the only way to create is
/// via `write_raise`.
#[derive(Debug)]
pub(crate) struct CompilerContext<'function> {
    functions: &'function mut Vec<UnasmFunction>,

    scope: ScopeContext<'function>,

    has_va_args: HasVaArgs,
    function: UnasmFunction,
}

impl CompilerContext<'_> {
    fn write_assignment<RegisterTy>(
        &mut self,
        mut targets: impl ExactSizeIterator<Item = impl RegisterMappable<RegisterTy>>,
        mut initializers: impl ExactSizeIterator<Item = impl CompileExpression>,
    ) -> Result<Option<OpError>, CompileError>
    where
        RegisterTy: Copy + InitRegister<RegisterTy>,
        UnasmRegister: From<RegisterTy>,
    {
        if initializers.len() == 0 {
            for dest in targets {
                dest.map(self)?.init_from_const(self, Constant::Nil);
            }
            return Ok(None);
        }

        let common_length = targets.len().min(initializers.len() - 1);

        for _ in 0..common_length {
            let (dest, init) = (
                targets.next().expect("Still in size of shortest iterator"),
                initializers
                    .next()
                    .expect("Still in size of shortest iterator"),
            );
            match init.compile(self)? {
                NodeOutput::Constant(value) => dest.map(self)?.init_from_const(self, value),
                NodeOutput::Register(source) => dest.map(self)?.init_from_reg(self, source),
                NodeOutput::ReturnValues => dest.map(self)?.init_from_ret(self),
                NodeOutput::VAStack => dest.map(self)?.init_from_va(self, 0),
                NodeOutput::Err(err) => return Ok(Some(err)),
            };
        }

        if targets.len() > 0 {
            debug_assert_eq!(initializers.len(), 1);

            match initializers
                .next()
                .map(|expr| expr.compile(self))
                .map_or(Ok(None), |res| res.map(Some))?
            {
                Some(NodeOutput::Constant(value)) => {
                    targets
                        .next()
                        .expect("Still in bounds for target length")
                        .map(self)?
                        .init_from_const(self, value);

                    for dest in targets {
                        dest.map(self)?.init_from_const(self, Constant::Nil);
                    }
                }
                Some(NodeOutput::Register(source)) => {
                    targets
                        .next()
                        .expect("Still in bounds for target length")
                        .map(self)?
                        .init_from_reg(self, source);

                    for dest in targets {
                        dest.map(self)?.init_from_const(self, Constant::Nil);
                    }
                }
                Some(NodeOutput::ReturnValues) => {
                    for dest in targets {
                        dest.map(self)?.init_from_ret(self);
                    }
                }
                Some(NodeOutput::VAStack) => {
                    for (index, dest) in targets.enumerate() {
                        dest.map(self)?.init_from_va(self, index);
                    }
                }
                _ => {
                    for dest in targets {
                        dest.map(self)?.init_from_const(self, Constant::Nil);
                    }
                }
            }
        } else {
            for init in initializers {
                if let NodeOutput::Err(err) = init.compile(self)? {
                    return Ok(Some(err));
                }
            }
        }

        Ok(None)
    }

    fn write_store_table(&mut self, table: UnasmRegister, index: UnasmOperand, value: NodeOutput) {
        match value {
            NodeOutput::Constant(c) => {
                self.emit(opcodes::Store::from((table, index, UnasmOperand::from(c))));
            }
            NodeOutput::Register(reg) => {
                self.emit(opcodes::Store::from((
                    table,
                    index,
                    UnasmOperand::from(reg),
                )));
            }
            NodeOutput::ReturnValues => {
                self.emit(opcodes::StoreRet::from((table, index)));
            }
            NodeOutput::VAStack => {
                self.emit(opcodes::StoreFromVa::from((table, index, 0)));
            }
            NodeOutput::Err(_) => unreachable!("Errors should already be handled."),
        }
    }
}

impl CompilerContext<'_> {
    /// Check if varargs are available in scope
    pub(crate) fn check_varargs(&self) -> Result<(), CompileError> {
        match self.has_va_args {
            HasVaArgs::None => Err(CompileError::NoVarArgsAvailable),
            HasVaArgs::Some => Ok(()),
        }
    }

    /// Get the current offset in the instruction stream.
    pub(crate) fn current_instruction(&self) -> usize {
        self.function.instructions.len()
    }

    /// Emit a new opcode to the current instruction stream. Returns the
    /// location in the instruction stream.
    pub(crate) fn emit(&mut self, opcode: impl Into<UnasmOp>) -> usize {
        self.function.instructions.push(opcode.into());
        self.current_instruction() - 1
    }

    /// Overwrite the instruction at location.
    pub(crate) fn overwrite(&mut self, location: usize, opcode: impl Into<UnasmOp>) {
        self.function.instructions[location] = opcode.into();
    }

    /// Map a new register for a local variable.
    pub(crate) fn new_local(
        &mut self,
        ident: Ident,
    ) -> Result<UninitRegister<OffsetRegister>, CompileError> {
        self.scope.new_local(ident)
    }

    /// Allocate a new anonymous register.
    pub(crate) fn new_anon_reg(&mut self) -> UninitRegister<AnonymousRegister> {
        self.scope.new_anonymous()
    }

    /// Allocate a sequence of anonymous registers.
    pub(crate) fn new_anon_reg_range(
        &mut self,
        size: usize,
    ) -> (
        usize,
        impl ExactSizeIterator<Item = UninitRegister<AnonymousRegister>>,
    ) {
        let start = self.scope.total_anons;
        let range = start..(start + size);
        for _ in range.clone() {
            let _ = self.new_anon_reg().no_init_needed();
        }

        (
            start,
            range.map(|idx| UninitRegister::from(AnonymousRegister::from(idx))),
        )
    }

    /// Instruct the compiler to emit a sequence of instructions corresponding
    /// to calling a function.
    /// Lookup the appropriate register for a specific identifier.
    pub(crate) fn read_variable(&mut self, ident: Ident) -> Result<UnasmRegister, CompileError> {
        Ok(self.scope.get_in_scope(ident)?.into())
    }

    /// Instruct the compiler to emit a sequence of instruction corresponding to
    /// raising an error with a compile-time known type.
    pub(crate) fn write_raise(&mut self, err: OpError) -> OpError {
        self.emit(opcodes::Raise::from(err));
        err
    }

    /// Create a new subcontext for compiling a function.
    pub(crate) fn function_subcontext(&mut self, has_va_args: HasVaArgs) -> CompilerContext {
        let scope = self.scope.subcontext();

        CompilerContext {
            functions: self.functions,
            has_va_args,
            function: UnasmFunction {
                anon_registers: 0,
                local_registers: 0,
                named_args: 0,
                instructions: Vec::default(),
            },

            scope,
        }
    }

    /// Finalize a subcontext and return the type metadata for it.
    pub(crate) fn complete_subcontext(self) -> TypeMeta {
        let Self {
            functions,
            mut function,
            scope,
            ..
        } = self;

        function.anon_registers = scope.total_anons;
        function.local_registers = scope.total_locals;

        let fn_id = functions.len();

        functions.push(function);

        TypeMeta::from(NonZeroUsize::try_from(fn_id + 1).ok())
    }

    /// Emit the instructions for a function.
    pub(crate) fn emit_fn(
        &mut self,
        params: impl ExactSizeIterator<Item = Ident>,
        body: impl ExactSizeIterator<Item = impl CompileStatement>,
        ret: Option<&impl CompileStatement>,
    ) -> Result<(), CompileError> {
        for param in params {
            // TODO(compiler-opt): Technically today this allocates an extra, unused
            // register for every duplicate identifier in the parameter list. It
            // still works fine though, because the number of registers is
            // correct.
            self.new_local(param)?.no_init_needed();
            self.function.named_args += 1;
        }

        for stat in body {
            stat.compile(self)?;
        }

        match ret {
            Some(ret) => ret.compile(self)?,
            None => {
                self.emit(opcodes::Op::Ret);
                None
            }
        };

        Ok(())
    }

    /// Instruct the compiler to compile a new block in its own subscope
    pub(crate) fn write_subscope(
        &mut self,
        mut body: impl ExactSizeIterator<Item = impl CompileStatement>,
        ret: Option<&impl CompileStatement>,
    ) -> Result<Option<OpError>, CompileError> {
        let mut result = Ok(None);

        // We don't want to allocate and merge in another sequence of instructions for
        // this function, but we do need a new subscope. Honestly this could be
        // done with traits, or a separate delegate object that handles this stuff, but
        // at that point things start to get hard to follow.
        // Here we just temporarily borrow ourselves, keep the current scope on the
        // stack and make a new scope which steals our function for a bit.
        take_mut::take(self, |context| {
            let CompilerContext {
                functions,
                mut scope,
                has_va_args,
                function,
            } = context;

            let (has_va_args, function, total_anons) = {
                let mut inner = CompilerContext {
                    functions,
                    scope: scope.subcontext(),
                    has_va_args,
                    function,
                };

                let pending = inner.function.instructions.len();
                inner.emit(opcodes::Raise {
                    err: OpError::ByteCodeError {
                        err: ByteCodeError::MissingScopeDescriptor,
                        offset: pending,
                    },
                });

                result = body
                    .try_for_each(|stat| stat.compile(&mut inner).map(|_| ()))
                    .and_then(|()| match ret {
                        Some(ret) => ret.compile(&mut inner),
                        None => Ok(None),
                    });

                if result.is_ok() {
                    inner.function.instructions[pending] = opcodes::ScopeDescriptor {
                        size: inner.scope.total_locals,
                    }
                    .into();
                    inner.emit(opcodes::Op::PopScope);
                }

                (inner.has_va_args, inner.function, inner.scope.total_anons)
            };

            scope.total_anons += total_anons;

            CompilerContext {
                functions,
                scope,
                has_va_args,
                function,
            }
        });

        result
    }

    /// Instruct the compiler to emit the instructions required to initialize a
    /// table.
    pub(crate) fn init_table(&mut self) -> UnasmRegister {
        self.new_anon_reg().init_alloc_table(self).into()
    }

    /// Instruct the compiler to emit the instructions required to set a value
    /// in a table based on an index.
    pub(crate) fn assign_to_array(
        &mut self,
        table: UnasmRegister,
        zero_based_index: usize,
        value: NodeOutput,
    ) -> Result<(), CompileError> {
        let index = UnasmOperand::from(i64::try_from(zero_based_index + 1).map_err(|_| {
            CompileError::TooManyTableEntries {
                max: i64::MAX as usize,
            }
        })?);

        self.write_store_table(table, index, value);

        Ok(())
    }

    /// Instruct the compiler to emit the instructions required copy a list of
    /// va arguments to the arraylike indicies of a table starting at
    /// `start_index`.
    pub(crate) fn copy_va_to_array(&mut self, table: UnasmRegister, zero_based_start_index: usize) {
        self.emit(opcodes::StoreAllFromVa::from((
            table,
            zero_based_start_index + 1,
        )));
    }

    /// Instruct the compiler to emit the instructions required copy a list of
    /// return values to the arraylike indicies of a table starting at
    /// `start_index`.
    pub(crate) fn copy_ret_to_array(
        &mut self,
        table: UnasmRegister,
        zero_based_start_index: usize,
    ) {
        self.emit(opcodes::StoreAllRet::from((
            table,
            zero_based_start_index + 1,
        )));
    }

    pub(crate) fn load_from_table(
        &mut self,
        table: UnasmRegister,
        index: impl CompileExpression,
    ) -> Result<Option<OpError>, CompileError> {
        let index = index.compile(self)?;
        match index {
            NodeOutput::Constant(c) => {
                self.emit(opcodes::Load::from((table, UnasmOperand::from(c))));
                Ok(None)
            }
            NodeOutput::Register(_) => todo!(),
            NodeOutput::ReturnValues => todo!(),
            NodeOutput::VAStack => todo!(),
            NodeOutput::Err(err) => Ok(Some(err)),
        }
    }

    /// Instruct the compiler to emit the instructions required to set a value
    /// in a table based on an expression.
    pub(crate) fn assign_to_table(
        &mut self,
        table: UnasmRegister,
        index: impl CompileExpression,
        value: impl CompileExpression,
    ) -> Result<Option<OpError>, CompileError> {
        let index = index.compile(self)?;
        let value = value.compile(self)?;

        match (index, value) {
            (
                NodeOutput::Constant(index),
                value @ NodeOutput::Constant(_)
                | value @ NodeOutput::Register(_)
                | value @ NodeOutput::ReturnValues
                | value @ NodeOutput::VAStack,
            ) => {
                self.write_store_table(table, index.into(), value);
                Ok(None)
            }
            (
                index @ NodeOutput::Register(_)
                | index @ NodeOutput::ReturnValues
                | index @ NodeOutput::VAStack,
                value @ NodeOutput::Constant(_)
                | value @ NodeOutput::Register(_)
                | value @ NodeOutput::ReturnValues
                | value @ NodeOutput::VAStack,
            ) => {
                let index = self.write_move_to_reg(index).into();
                self.write_store_table(table, index, value);

                Ok(None)
            }
            (NodeOutput::Err(err), _) | (_, NodeOutput::Err(err)) => Ok(Some(err)),
        }
    }

    /// Instruct the compiler to emit a sequence of instruction corresponding to
    /// returning some number of values from a function.
    pub(crate) fn write_ret_stack_sequence(
        &mut self,
        mut outputs: impl ExactSizeIterator<Item = impl CompileExpression>,
    ) -> Result<Option<OpError>, CompileError> {
        if outputs.len() == 0 {
            self.emit(opcodes::Op::Ret);
            return Ok(None);
        }

        let normal_retc = outputs.len() - 1;

        for _ in 0..normal_retc {
            match outputs
                .next()
                .expect("Still in bounds for outputs")
                .compile(self)?
            {
                NodeOutput::Constant(c) => {
                    self.emit(opcodes::SetRet::from(UnasmOperand::from(c)));
                }
                NodeOutput::Register(register) => {
                    self.emit(opcodes::SetRet::from(UnasmOperand::from(register)));
                }
                NodeOutput::Err(err) => {
                    return Ok(Some(err));
                }
                NodeOutput::ReturnValues => {
                    self.emit(opcodes::Op::SetRetFromRet0);
                }
                NodeOutput::VAStack => {
                    self.emit(opcodes::Op::SetRetVa0);
                }
            }
        }

        match outputs
            .next()
            .expect("Still in bounds for outputs")
            .compile(self)?
        {
            NodeOutput::Constant(c) => {
                self.emit(opcodes::SetRet::from(UnasmOperand::from(c)));
                self.emit(opcodes::Op::Ret);
            }
            NodeOutput::Register(register) => {
                self.emit(opcodes::SetRet::from(UnasmOperand::from(register)));
                self.emit(opcodes::Op::Ret);
            }
            NodeOutput::ReturnValues => {
                self.emit(opcodes::Op::CopyRetFromRetAndRet);
            }
            NodeOutput::VAStack => {
                self.emit(opcodes::Op::CopyRetFromVaAndRet);
            }
            NodeOutput::Err(err) => {
                return Ok(Some(err));
            }
        }

        Ok(None)
    }

    /// Instruct the compiler to emit a sequence of instructions for local
    /// variable initialization.
    pub(crate) fn write_assign_all_locals(
        &mut self,
        targets: impl ExactSizeIterator<Item = LocalVariableTarget>,
        initializers: impl ExactSizeIterator<Item = impl CompileExpression>,
    ) -> Result<Option<OpError>, CompileError> {
        self.write_assignment(targets, initializers)
    }

    /// Instruct the compiler to emit a sequence of instructions for variable
    /// initialization.
    /// Note that LUA 5.4 has special, undocumented, rules for how multiple
    /// return values from a function & multiple assignments interact. If
    /// you have a function with multiple return values in the middle of a
    /// list of initializers, only the first value returned from that function
    /// will be used. If a function with multiple return values is the _last_
    /// item in the list, it will yield up to all of its values to initialize
    /// each variable.
    pub(crate) fn write_assign_all(
        &mut self,
        targets: impl ExactSizeIterator<Item = VariableTarget>,
        initializers: impl ExactSizeIterator<Item = impl CompileExpression>,
    ) -> Result<Option<OpError>, CompileError> {
        self.write_assignment(targets, initializers)
    }

    /// Instruct the compiler to emit a sequence of instructions corresponding
    /// to a binary operation on the result of two nodes.
    pub(crate) fn write_binop<Op, Lhs, Rhs, ConstEval>(
        &mut self,
        lhs: Lhs,
        rhs: Rhs,
        consteval: ConstEval,
    ) -> Result<NodeOutput, CompileError>
    where
        Op: From<(UnasmRegister, UnasmOperand)> + Into<UnasmOp>,
        Lhs: CompileExpression,
        Rhs: CompileExpression,
        ConstEval: FnOnce(Constant, Constant) -> Result<Constant, OpError>,
    {
        let lhs = lhs.compile(self)?;
        let rhs = rhs.compile(self)?;

        // TODO(compiler-opt): Technically, more efficient use could be made of
        // registers here by checking if the operation is commutative and
        // swapping constants to the right or existing anonymous registers to
        // the left.
        match (lhs, rhs) {
            (NodeOutput::Constant(lhs), NodeOutput::Constant(rhs)) => match consteval(lhs, rhs) {
                Ok(constant) => Ok(NodeOutput::Constant(constant)),
                Err(err) => Ok(NodeOutput::Err(self.write_raise(err))),
            },
            (
                lhs @ NodeOutput::Register(_)
                | lhs @ NodeOutput::ReturnValues
                | lhs @ NodeOutput::VAStack,
                NodeOutput::Constant(rhs),
            ) => {
                let lhs = self.write_move_to_reg(lhs);

                self.emit(Op::from((lhs.into(), rhs.into())));

                Ok(NodeOutput::Register(lhs.into()))
            }
            (
                lhs @ NodeOutput::Constant(_)
                | lhs @ NodeOutput::Register(_)
                | lhs @ NodeOutput::ReturnValues
                | lhs @ NodeOutput::VAStack,
                rhs @ NodeOutput::ReturnValues | rhs @ NodeOutput::VAStack,
            ) => {
                let lhs = self.write_move_to_reg(lhs);
                let rhs = self.write_move_to_reg(rhs);

                self.emit(Op::from((lhs.into(), rhs.into())));

                Ok(NodeOutput::Register(lhs.into()))
            }
            (
                lhs @ NodeOutput::Constant(_)
                | lhs @ NodeOutput::Register(_)
                | lhs @ NodeOutput::ReturnValues
                | lhs @ NodeOutput::VAStack,
                NodeOutput::Register(rhs),
            ) => {
                let lhs = self.write_move_to_reg(lhs);

                self.emit(Op::from((lhs.into(), rhs.into())));

                Ok(NodeOutput::Register(lhs.into()))
            }
            (NodeOutput::Err(err), _) | (_, NodeOutput::Err(err)) => Ok(NodeOutput::Err(err)),
        }
    }

    pub(crate) fn write_unary_op<Op, Operand, ConstEval>(
        &mut self,
        operand: Operand,
        consteval: ConstEval,
    ) -> Result<NodeOutput, CompileError>
    where
        Op: From<UnasmRegister> + Into<UnasmOp>,
        Operand: CompileExpression,
        ConstEval: FnOnce(Constant) -> Result<Constant, OpError>,
    {
        match operand.compile(self)? {
            NodeOutput::Constant(c) => match consteval(c) {
                Ok(val) => Ok(NodeOutput::Constant(val)),
                Err(err) => Ok(NodeOutput::Err(self.write_raise(err))),
            },
            reg @ NodeOutput::Register(_)
            | reg @ NodeOutput::ReturnValues
            | reg @ NodeOutput::VAStack => {
                let reg = self.write_move_to_reg(reg);

                self.emit(Op::from(reg.into()));

                Ok(NodeOutput::Register(reg.into()))
            }
            NodeOutput::Err(err) => Ok(NodeOutput::Err(err)),
        }
    }

    pub(crate) fn write_move_to_reg(&mut self, value: NodeOutput) -> AnonymousRegister {
        if let NodeOutput::Register(UnasmRegister::Immediate(reg)) = value {
            reg
        } else {
            let reg = self.new_anon_reg();
            match value {
                NodeOutput::Constant(c) => reg.init_from_const(self, c),
                NodeOutput::Register(r) => reg.init_from_reg(self, r),
                NodeOutput::ReturnValues => reg.init_from_ret(self),
                NodeOutput::VAStack => reg.init_from_va(self, 0),
                NodeOutput::Err(_) => {
                    unreachable!("Errors should not be handled by storing them in registers.")
                }
            }
        }
    }
}
