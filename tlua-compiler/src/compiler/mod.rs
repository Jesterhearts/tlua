use std::ops::{
    Deref,
    DerefMut,
};

use derive_more::From;
use tlua_bytecode::{
    opcodes,
    ByteCodeError,
    Constant,
    FuncId,
    OpError,
    Truthy,
};
use tlua_parser::ast::{
    block::Block,
    identifiers::Ident,
};

use crate::{
    Chunk,
    CompileError,
    CompileExpression,
    CompileStatement,
    NodeOutput,
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
enum HasVaArgs {
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

trait RegisterMappable<InitTy, RegisterTy = InitTy>: Sized
where
    RegisterTy: Copy,
    InitTy: InitRegister<RegisterTy>,
    UnasmRegister: From<RegisterTy>,
{
    fn map(self, compiler: &mut CompilerContext) -> Result<InitTy, CompileError>;
}

impl RegisterMappable<LocalRegister, LocalRegister> for LocalVariableTarget {
    fn map(self, compiler: &mut CompilerContext) -> Result<LocalRegister, CompileError> {
        match self {
            LocalVariableTarget::Mutable(ident) => {
                Ok(compiler.scope.new_local(ident)?.no_init_needed().into())
            }
            LocalVariableTarget::Constant(ident) => {
                Ok(compiler.scope.new_constant(ident)?.no_init_needed().into())
            }
            LocalVariableTarget::Closable(_) => todo!(),
        }
    }
}

impl RegisterMappable<UnasmRegister, UnasmRegister> for VariableTarget {
    fn map(self, compiler: &mut CompilerContext) -> Result<UnasmRegister, CompileError> {
        match self {
            VariableTarget::Ident(ident) => Ok(compiler.scope.get_in_scope(ident)?.into()),
            VariableTarget::Register(reg) => Ok(reg),
        }
    }
}

trait InitRegister<RegisterTy>: Sized
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
        compiler.write(opcodes::MapRet::from(UnasmRegister::from(register)));
        register
    }

    /// Indicate that the register should be initialized to a constant. If the
    /// constant is always nil, please use init_from_nil.
    fn init_from_const(self, compiler: &mut CompilerContext, value: Constant) -> RegisterTy {
        let register = self.no_init_needed();
        compiler.write(opcodes::Set::from((UnasmRegister::from(register), value)));
        register
    }

    /// Indicate the the register should be initialized by allocating a
    /// function.
    fn init_alloc_fn(self, compiler: &mut CompilerContext, value: FuncId) -> RegisterTy {
        let register = self.no_init_needed();
        compiler.write(opcodes::AllocFunc::from((
            UnasmRegister::from(register),
            value,
        )));
        register
    }

    /// Indicate the the register shoudl be initialized by allocating a table
    fn init_alloc_table(self, compiler: &mut CompilerContext) -> RegisterTy {
        let register = self.no_init_needed();
        compiler.write(opcodes::AllocTable::from(UnasmRegister::from(register)));
        register
    }

    /// Indicate that the register should be initialized from another register.
    fn init_from_reg(self, compiler: &mut CompilerContext, other: UnasmRegister) -> RegisterTy {
        let register = self.no_init_needed();
        compiler.write(opcodes::SetIndirect::from((
            UnasmRegister::from(register),
            other,
        )));
        register
    }

    fn init_from_va(self, compiler: &mut CompilerContext, index: usize) -> RegisterTy {
        let register = self.no_init_needed();
        compiler.write(opcodes::SetFromVa::from((
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

impl InitRegister<LocalRegister> for LocalRegister {
    fn no_init_needed(self) -> LocalRegister {
        self
    }
}

#[derive(Debug, From)]
#[must_use]
struct UninitRegister<RegisterTy> {
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
    fn function_subcontext(&mut self, has_va_args: HasVaArgs) -> CompilerContext {
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

    fn complete(self) -> FuncId {
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

        FuncId(fn_id)
    }

    fn write_fn(
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
            self.scope.new_local(param)?.no_init_needed();
            self.function.named_args += 1;
        }

        for stat in body {
            stat.compile(self)?;
        }

        match ret {
            Some(ret) => ret.compile(self)?,
            None => {
                self.write(opcodes::Op::Ret);
                None
            }
        };

        Ok(())
    }

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

    fn write_move_to_reg(&mut self, value: NodeOutput) -> AnonymousRegister {
        if let NodeOutput::Register(UnasmRegister::Anonymous(reg)) = value {
            reg
        } else {
            let reg = self.scope.new_anonymous();
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

    fn write_store_table_constant_idx(
        &mut self,
        table: UnasmRegister,
        index: Constant,
        value: NodeOutput,
    ) {
        match value {
            NodeOutput::Constant(c) => {
                self.write(opcodes::StoreConstant::from((table, index, c)));
            }
            NodeOutput::Register(reg) => {
                self.write(opcodes::Store::from((table, index, reg)));
            }
            NodeOutput::ReturnValues => todo!(),
            NodeOutput::VAStack => {
                self.write(opcodes::StoreFromVa::from((table, index, 0)));
            }
            NodeOutput::Err(_) => unreachable!("Errors should already be handled."),
        }
    }

    fn write_store_table_indirect(
        &mut self,
        table: UnasmRegister,
        index: UnasmRegister,
        value: NodeOutput,
    ) {
        match value {
            NodeOutput::Constant(c) => {
                self.write(opcodes::StoreConstantIndirect::from((table, index, c)));
            }
            NodeOutput::Register(reg) => {
                self.write(opcodes::StoreIndirect::from((table, index, reg)));
            }
            NodeOutput::ReturnValues => todo!(),
            NodeOutput::VAStack => {
                self.write(opcodes::StoreFromVaIndirect::from((table, index, 0)));
            }
            NodeOutput::Err(_) => unreachable!("Errors should already be handled."),
        }
    }

    fn write(&mut self, opcode: impl Into<UnasmOp>) {
        self.function.instructions.push(opcode.into());
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

    /// Instruct the compiler to emit a sequence of instructions corresponding
    /// to calling a function.
    pub(crate) fn write_call(
        &mut self,
        target: UnasmRegister,
        mut args: impl ExactSizeIterator<Item = impl CompileExpression>,
    ) -> Result<Option<OpError>, CompileError> {
        enum ArgSrc {
            Const(Constant),
            Register(UnasmRegister),
            Va0,
        }

        if args.len() == 0 {
            // No arguments, just call.
            self.write(opcodes::StartCall::from(target));
            self.write(opcodes::Op::DoCall);
            return Ok(None);
        }

        let regular_argc = args.len() - 1;

        let mut arg_srcs = Vec::with_capacity(args.len());

        for _ in 0..regular_argc {
            match args
                .next()
                .expect("Still in bounds for args")
                .compile(self)?
            {
                NodeOutput::Constant(c) => arg_srcs.push(ArgSrc::Const(c)),
                NodeOutput::Register(r) => arg_srcs.push(ArgSrc::Register(r)),
                NodeOutput::ReturnValues => {
                    let dest = self.scope.new_anonymous();
                    let dest = dest.init_from_ret(self);
                    arg_srcs.push(ArgSrc::Register(dest.into()));
                }
                NodeOutput::VAStack => {
                    arg_srcs.push(ArgSrc::Va0);
                }
                NodeOutput::Err(err) => return Ok(Some(err)),
            };
        }

        let write_args = |compiler: &mut CompilerContext, arg_srcs: Vec<ArgSrc>| {
            for arg in arg_srcs.into_iter() {
                match arg {
                    ArgSrc::Const(constant) => compiler.write(opcodes::MapArg::from(constant)),
                    ArgSrc::Register(register) => {
                        compiler.write(opcodes::MapArgIndirect::from(register))
                    }
                    ArgSrc::Va0 => compiler.write(opcodes::Op::MapVa0),
                }
            }
        };

        // Process the last argument in the list
        match args.next().expect("Still in bounds args").compile(self)? {
            NodeOutput::Constant(c) => {
                arg_srcs.push(ArgSrc::Const(c));
            }
            NodeOutput::Register(r) => {
                arg_srcs.push(ArgSrc::Register(r));
            }
            NodeOutput::ReturnValues => {
                self.write(opcodes::StartCallExtending::from(target));
                write_args(self, arg_srcs);
                return Ok(None);
            }
            NodeOutput::VAStack => {
                self.write(opcodes::StartCall::from(target));
                write_args(self, arg_srcs);
                self.write(opcodes::Op::MapVarArgsAndDoCall);
                return Ok(None);
            }
            NodeOutput::Err(err) => return Ok(Some(err)),
        }

        self.write(opcodes::StartCall::from(target));
        write_args(self, arg_srcs);
        self.write(opcodes::Op::DoCall);
        Ok(None)
    }

    /// Lookup the appropriate register for a specific identifier.
    pub(crate) fn read_variable(&mut self, ident: Ident) -> Result<UnasmRegister, CompileError> {
        Ok(self.scope.get_in_scope(ident)?.into())
    }

    /// Instruct the compiler to emit a sequence of instruction corresponding to
    /// raising an error with a compile-time known type.
    pub(crate) fn write_raise(&mut self, err: OpError) {
        self.write(opcodes::Raise::from(err));
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
                inner.write(opcodes::Raise {
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
                    inner.write(opcodes::Op::PopScope);
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

    pub(crate) fn write_if_sequence(
        &mut self,
        conditions: impl Iterator<Item = impl CompileExpression>,
        bodies: impl Iterator<Item = impl CompileStatement>,
    ) -> Result<Option<OpError>, CompileError> {
        let mut conditions = conditions.peekable();
        let mut bodies = bodies.peekable();

        let mut pending_exit_location = None;
        let cleanup_pending = |pending, compiler: &mut CompilerContext| {
            if let Some(pending) = pending {
                compiler.function.instructions[pending] = opcodes::Jump {
                    target: compiler.function.instructions.len(),
                }
                .into();
            }
        };

        let cleanup_add_pending_block_exit = |pending, compiler: &mut CompilerContext| {
            cleanup_pending(pending, compiler);

            let pending = compiler.function.instructions.len();
            compiler.write(opcodes::Raise {
                err: OpError::ByteCodeError {
                    err: ByteCodeError::MissingJump,
                    offset: pending,
                },
            });
            pending
        };

        while let (Some(_), Some(_)) = (conditions.peek(), bodies.peek()) {
            let (cond, body) = (
                conditions.next().expect("Saw a cond"),
                bodies.next().expect("Saw a body"),
            );

            let cond_value = cond.compile(self)?;

            if let NodeOutput::Constant(c) = cond_value {
                if c.as_bool() {
                    // No other branches are reachable.
                    body.compile(self)?;
                    cleanup_pending(pending_exit_location, self);
                    return Ok(None);
                } else {
                    // The body is statically unreachable, skip it.
                    continue;
                }
            } else if let NodeOutput::Err(err) = cond_value {
                // If evaluating the condition would statically raise, we can skip compiling the
                // rest of the sequence, since it's unreachable.
                return Ok(Some(err));
            }

            let jump_location = self.function.instructions.len();
            self.write(opcodes::Raise {
                err: OpError::ByteCodeError {
                    err: ByteCodeError::MissingJump,
                    offset: jump_location,
                },
            });

            body.compile(self)?;

            pending_exit_location =
                Some(cleanup_add_pending_block_exit(pending_exit_location, self));

            self.function.instructions[jump_location] = match cond_value {
                NodeOutput::Register(reg) => opcodes::JumpNot {
                    cond: reg,
                    target: self.function.instructions.len(),
                }
                .into(),
                NodeOutput::ReturnValues => opcodes::JumpNotRet0 {
                    target: self.function.instructions.len(),
                }
                .into(),
                NodeOutput::VAStack => opcodes::JumpNotVa0 {
                    target: self.function.instructions.len(),
                }
                .into(),
                NodeOutput::Err(_) | NodeOutput::Constant(_) => {
                    unreachable!("Constant conditions should already be handled.")
                }
            }
        }

        debug_assert!(conditions.next().is_none());
        if let Some(last) = bodies.next() {
            last.compile(self)?;
        }
        cleanup_pending(pending_exit_location, self);

        Ok(None)
    }

    /// Instruct the compiler to compile a new global function.
    pub(crate) fn write_global_fn(
        &mut self,
        params: impl ExactSizeIterator<Item = Ident>,
        body: impl ExactSizeIterator<Item = impl CompileStatement>,
        ret: Option<&impl CompileStatement>,
    ) -> Result<FuncId, CompileError> {
        let mut context = self.function_subcontext(HasVaArgs::None);

        context.write_fn(params, body, ret)?;

        Ok(context.complete())
    }

    /// Instruct the compiler to compile a new global function with variadic
    /// arguments.
    pub(crate) fn write_va_global_fn(
        &mut self,
        params: impl ExactSizeIterator<Item = Ident>,
        body: impl ExactSizeIterator<Item = impl CompileStatement>,
        ret: Option<&impl CompileStatement>,
    ) -> Result<FuncId, CompileError> {
        let mut context = self.function_subcontext(HasVaArgs::Some);

        context.write_fn(params, body, ret)?;

        Ok(context.complete())
    }

    /// Instruct the compiler to compile a new local function bound to `name`.
    pub(crate) fn write_local_fn(
        &mut self,
        name: Ident,
        params: impl ExactSizeIterator<Item = Ident>,
        body: impl ExactSizeIterator<Item = impl CompileStatement>,
        ret: Option<&impl CompileStatement>,
    ) -> Result<(), CompileError> {
        // This variable will be in scope for all child scopes :(
        // So we have to allocate a register for it here before compiling the function
        // body.
        let register = self.scope.new_local(name)?;

        let mut context = self.function_subcontext(HasVaArgs::None);

        context.write_fn(params, body, ret)?;

        let fn_id = context.complete();

        // Because this is a local function declaration, we know we're the first write
        // to it in scope. We had to have the register already allocated though so it
        // could be in scope during compilation of child scopes.
        register.init_alloc_fn(self, fn_id);

        Ok(())
    }

    /// Instruct the compiler to compile a new local function bound to `name`
    /// with variadic arguments.
    pub(crate) fn write_va_local_fn(
        &mut self,
        name: Ident,
        params: impl ExactSizeIterator<Item = Ident>,
        body: impl ExactSizeIterator<Item = impl CompileStatement>,
        ret: Option<&impl CompileStatement>,
    ) -> Result<(), CompileError> {
        let register = self.scope.new_local(name)?;

        let mut context = self.function_subcontext(HasVaArgs::Some);

        context.write_fn(params, body, ret)?;

        let fn_id = context.complete();

        register.init_alloc_fn(self, fn_id);

        Ok(())
    }

    /// Instruct the compiler to emit the instructions required to initialize a
    /// table.
    pub(crate) fn init_table(&mut self) -> UnasmRegister {
        self.scope.new_anonymous().init_alloc_table(self).into()
    }

    /// Instruct the compiler to emit the instructions required to set a value
    /// in a table based on an index.
    pub(crate) fn assign_to_array(
        &mut self,
        table: UnasmRegister,
        zero_based_index: usize,
        value: NodeOutput,
    ) -> Result<(), CompileError> {
        let index = Constant::from(i64::try_from(zero_based_index + 1).map_err(|_| {
            CompileError::TooManyTableEntries {
                max: i64::MAX as usize,
            }
        })?);

        self.write_store_table_constant_idx(table, index, value);

        Ok(())
    }

    /// Instruct the compiler to emit the instructions required copy a list of
    /// va arguments to the arraylike indicies of a table starting at
    /// `start_index`.
    pub(crate) fn copy_va_to_array(&mut self, table: UnasmRegister, zero_based_start_index: usize) {
        self.write(opcodes::StoreAllFromVa::from((
            table,
            zero_based_start_index + 1,
        )));
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
                self.write_store_table_constant_idx(table, index, value);
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
                self.write_store_table_indirect(table, index, value);

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
            self.write(opcodes::Op::Ret);
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
                    self.write(opcodes::SetRet::from(c));
                }
                NodeOutput::Register(register) => {
                    self.write(opcodes::SetRetIndirect::from(register));
                }
                NodeOutput::Err(err) => {
                    return Ok(Some(err));
                }
                NodeOutput::ReturnValues => {
                    self.write(opcodes::Op::SetRetFromRet0);
                }
                NodeOutput::VAStack => self.write(opcodes::Op::SetRetVa0),
            }
        }

        match outputs
            .next()
            .expect("Still in bounds for outputs")
            .compile(self)?
        {
            NodeOutput::Constant(c) => {
                self.write(opcodes::SetRet::from(c));
                self.write(opcodes::Op::Ret)
            }
            NodeOutput::Register(register) => {
                self.write(opcodes::SetRetIndirect::from(register));
                self.write(opcodes::Op::Ret)
            }
            NodeOutput::ReturnValues => {
                self.write(opcodes::Op::CopyRetFromRetAndRet);
            }
            NodeOutput::VAStack => self.write(opcodes::Op::CopyRetFromVaAndRet),
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
    pub(crate) fn write_binop<Op, OpIndirect, Lhs, Rhs, ConstEval>(
        &mut self,
        lhs: Lhs,
        rhs: Rhs,
        consteval: ConstEval,
    ) -> Result<NodeOutput, CompileError>
    where
        Op: From<(UnasmRegister, Constant)> + Into<UnasmOp>,
        OpIndirect: From<(UnasmRegister, UnasmRegister)> + Into<UnasmOp>,
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
                Err(err) => {
                    self.write_raise(err);
                    Ok(NodeOutput::Err(err))
                }
            },
            (
                lhs @ NodeOutput::Register(_)
                | lhs @ NodeOutput::ReturnValues
                | lhs @ NodeOutput::VAStack,
                NodeOutput::Constant(rhs),
            ) => {
                let lhs = self.write_move_to_reg(lhs);

                self.write(Op::from((lhs.into(), rhs)));

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

                self.write(OpIndirect::from((lhs.into(), rhs.into())));

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

                self.write(OpIndirect::from((lhs.into(), rhs)));

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
                Err(err) => {
                    self.write_raise(err);
                    Ok(NodeOutput::Err(err))
                }
            },
            reg @ NodeOutput::Register(_)
            | reg @ NodeOutput::ReturnValues
            | reg @ NodeOutput::VAStack => {
                let reg = self.write_move_to_reg(reg);

                self.write(Op::from(reg.into()));

                Ok(NodeOutput::Register(reg.into()))
            }
            NodeOutput::Err(err) => Ok(NodeOutput::Err(err)),
        }
    }
}
