use derive_more::From;
use tlua_bytecode::{
    opcodes,
    AnonymousRegister,
    ByteCodeError,
    OpError,
};
use tlua_parser::ast::{
    block::Block,
    identifiers::Ident,
};

use crate::{
    block::emit_block,
    constant::Constant,
    BuiltinType,
    Chunk,
    CompileError,
    CompileExpression,
    CompileStatement,
    FuncId,
    NodeOutput,
};

mod scope;
pub(super) mod unasm;

use self::{
    scope::*,
    unasm::*,
};

#[derive(Debug, Clone, Copy)]
pub(crate) enum HasVaArgs {
    None,
    Some,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) enum LabelId {
    Named(Ident),
    If { id: usize },
    Loop { id: usize },
}

#[derive(Debug, Default)]
pub(crate) struct Compiler {
    scope: RootScope,
    functions: Vec<UnasmFunction>,
}

impl Compiler {
    pub(crate) fn compile_ast(mut self, ast: Block) -> Result<Chunk, CompileError> {
        let main = self.emit_in_main(|context| emit_block(context, &ast).map(|_| ()))?;

        Ok(self.into_chunk(main))
    }

    pub(super) fn emit_in_main(
        &mut self,
        mut emitter: impl FnMut(&mut CompilerContext) -> Result<(), CompileError>,
    ) -> Result<UnasmFunction, CompileError> {
        let mut main = self.scope.main_function();
        {
            let scope = main.start();
            let mut context = CompilerContext {
                functions: &mut self.functions,
                scope,
                has_va_args: HasVaArgs::None,
            };

            let () = emitter(&mut context)?;
        }

        Ok(main.complete())
    }

    fn into_chunk(self, main: UnasmFunction) -> Chunk {
        let Self { functions, scope } = self;

        Chunk {
            globals_map: scope
                .into_globals()
                .into_iter()
                .map(|(global, reg)| {
                    debug_assert_eq!(reg.source_scope_depth, GLOBAL_SCOPE);
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

pub(crate) trait InitRegister<RegisterTy = Self>: Sized {
    /// Indicate that the register should always init to nil, and needs no
    /// special handling.
    fn no_init_needed(self) -> RegisterTy;

    /// Initialize the register from node output.
    fn init_from_node_output(
        self,
        compiler: &mut CompilerContext,
        value: NodeOutput,
    ) -> RegisterTy {
        match value {
            NodeOutput::Constant(value) => self.init_from_const(compiler, value),
            NodeOutput::Immediate(source) => self.init_from_anon_reg(compiler, source),
            NodeOutput::MappedRegister(source) => self.init_from_mapped_reg(compiler, source),
            NodeOutput::TableEntry { table, index } => {
                self.init_from_table_entry(compiler, table, index)
            }
            NodeOutput::ReturnValues => self.init_from_ret(compiler),
            NodeOutput::VAStack => self.init_from_va(compiler, 0),
            NodeOutput::Err(_) => self.no_init_needed(),
        }
    }

    /// Indicate that the register should be initialized from a return value.
    fn init_from_ret(self, compiler: &mut CompilerContext) -> RegisterTy;

    /// Indicate that the register should be initialized to a constant. If the
    /// constant is always nil, please use init_from_nil.
    fn init_from_const(self, compiler: &mut CompilerContext, value: Constant) -> RegisterTy;

    /// Indicate the the register should be initialized by allocating a
    /// function.
    fn init_alloc_fn(self, compiler: &mut CompilerContext, value: FuncId) -> RegisterTy;

    /// Indicate the the register shoudl be initialized by allocating a table
    fn init_alloc_table(self, compiler: &mut CompilerContext) -> RegisterTy;

    /// Indicate that the register should be initialized from another register.
    fn init_from_anon_reg(
        self,
        compiler: &mut CompilerContext,
        other: AnonymousRegister,
    ) -> RegisterTy;

    /// Indicate that the register should be initialized from another register.
    fn init_from_mapped_reg(
        self,
        compiler: &mut CompilerContext,
        other: MappedLocalRegister,
    ) -> RegisterTy;

    /// Indicate that the register should be initialized from a table entry
    fn init_from_table_entry(
        self,
        compiler: &mut CompilerContext,
        table: AnonymousRegister,
        index: AnonymousRegister,
    ) -> RegisterTy;

    /// Indicate that the register should be initialized from a variadic
    /// argument;
    fn init_from_va(self, compiler: &mut CompilerContext, index: usize) -> RegisterTy;
}

impl InitRegister for AnonymousRegister {
    fn no_init_needed(self) -> Self {
        self
    }

    fn init_from_ret(self, compiler: &mut CompilerContext) -> Self {
        let reg = self.no_init_needed();
        compiler.emit(opcodes::ConsumeRetRange::from((usize::from(reg), 1)));
        reg
    }

    fn init_from_const(self, compiler: &mut CompilerContext, value: Constant) -> Self {
        let reg = self.no_init_needed();
        compiler.emit(opcodes::LoadConstant::from((reg, value.into())));
        reg
    }

    fn init_alloc_fn(self, compiler: &mut CompilerContext, value: FuncId) -> Self {
        let reg = self.no_init_needed();
        compiler.emit(opcodes::Alloc::from((
            reg,
            BuiltinType::Function(value).into(),
        )));

        reg
    }

    fn init_alloc_table(self, compiler: &mut CompilerContext) -> Self {
        let reg = self.no_init_needed();
        compiler.emit(opcodes::Alloc::from((reg, BuiltinType::Table.into())));

        reg
    }

    fn init_from_anon_reg(self, compiler: &mut CompilerContext, other: AnonymousRegister) -> Self {
        let reg = self.no_init_needed();
        if other != reg {
            compiler.emit(opcodes::DuplicateRegister::from((reg, other)));
        }
        reg
    }

    fn init_from_mapped_reg(
        self,
        compiler: &mut CompilerContext,
        other: MappedLocalRegister,
    ) -> Self {
        let reg = self.no_init_needed();
        compiler.emit(opcodes::LoadRegister::from((reg, other)));
        reg
    }

    fn init_from_table_entry(
        self,
        compiler: &mut CompilerContext,
        table: AnonymousRegister,
        index: AnonymousRegister,
    ) -> Self {
        let reg = self.no_init_needed();
        compiler.emit(opcodes::Lookup::from((reg, table, index)));
        reg
    }

    fn init_from_va(self, compiler: &mut CompilerContext, index: usize) -> Self {
        let reg = self.no_init_needed();
        compiler.emit(opcodes::LoadVa::from((usize::from(reg), index, 1)));
        reg
    }
}

impl InitRegister for MappedLocalRegister {
    fn no_init_needed(self) -> Self {
        self
    }

    fn init_from_ret(self, compiler: &mut CompilerContext) -> Self {
        let anon = compiler.new_anon_reg().init_from_ret(compiler);
        self.init_from_anon_reg(compiler, anon)
    }

    fn init_from_const(self, compiler: &mut CompilerContext, value: Constant) -> Self {
        let anon = compiler.new_anon_reg().init_from_const(compiler, value);
        self.init_from_anon_reg(compiler, anon)
    }

    fn init_alloc_fn(self, compiler: &mut CompilerContext, value: FuncId) -> Self {
        let anon = compiler.new_anon_reg().init_alloc_fn(compiler, value);
        self.init_from_anon_reg(compiler, anon)
    }

    fn init_alloc_table(self, compiler: &mut CompilerContext) -> Self {
        let anon = compiler.new_anon_reg().init_alloc_table(compiler);
        self.init_from_anon_reg(compiler, anon)
    }

    fn init_from_anon_reg(self, compiler: &mut CompilerContext, other: AnonymousRegister) -> Self {
        let reg = self.no_init_needed();
        compiler.emit(opcodes::Store::from((reg, other)));
        reg
    }

    fn init_from_mapped_reg(
        self,
        compiler: &mut CompilerContext,
        other: MappedLocalRegister,
    ) -> Self {
        let anon = compiler
            .new_anon_reg()
            .init_from_mapped_reg(compiler, other);
        self.init_from_anon_reg(compiler, anon)
    }

    fn init_from_table_entry(
        self,
        compiler: &mut CompilerContext,
        table: AnonymousRegister,
        index: AnonymousRegister,
    ) -> Self {
        let anon = compiler
            .new_anon_reg()
            .init_from_table_entry(compiler, table, index);
        self.init_from_anon_reg(compiler, anon)
    }

    fn init_from_va(self, compiler: &mut CompilerContext, index: usize) -> Self {
        let anon = compiler.new_anon_reg().init_from_va(compiler, index);
        self.init_from_anon_reg(compiler, anon)
    }
}

#[derive(Debug, From)]
#[must_use]
pub(crate) struct UninitRegister<RegisterTy> {
    register: RegisterTy,
}

impl<RegisterTy> InitRegister<RegisterTy> for UninitRegister<RegisterTy>
where
    RegisterTy: InitRegister,
{
    fn no_init_needed(self) -> RegisterTy {
        self.register
    }

    fn init_from_ret(self, compiler: &mut CompilerContext) -> RegisterTy {
        self.register.init_from_ret(compiler)
    }

    fn init_from_const(self, compiler: &mut CompilerContext, value: Constant) -> RegisterTy {
        self.register.init_from_const(compiler, value)
    }

    fn init_alloc_fn(self, compiler: &mut CompilerContext, value: FuncId) -> RegisterTy {
        self.register.init_alloc_fn(compiler, value)
    }

    fn init_alloc_table(self, compiler: &mut CompilerContext) -> RegisterTy {
        self.register.init_alloc_table(compiler)
    }

    fn init_from_anon_reg(
        self,
        compiler: &mut CompilerContext,
        other: AnonymousRegister,
    ) -> RegisterTy {
        self.register.init_from_anon_reg(compiler, other)
    }

    fn init_from_mapped_reg(
        self,
        compiler: &mut CompilerContext,
        other: MappedLocalRegister,
    ) -> RegisterTy {
        self.register.init_from_mapped_reg(compiler, other)
    }

    fn init_from_table_entry(
        self,
        compiler: &mut CompilerContext,
        table: AnonymousRegister,
        index: AnonymousRegister,
    ) -> RegisterTy {
        self.register.init_from_table_entry(compiler, table, index)
    }

    fn init_from_va(self, compiler: &mut CompilerContext, index: usize) -> RegisterTy {
        self.register.init_from_va(compiler, index)
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
pub(crate) struct CompilerContext<'context, 'block, 'function> {
    functions: &'context mut Vec<UnasmFunction>,

    scope: BlockScope<'block, 'function>,

    has_va_args: HasVaArgs,
}

impl CompilerContext<'_, '_, '_> {
    /// Check if varargs are available in scope
    pub(crate) fn check_varargs(&self) -> Result<(), CompileError> {
        match self.has_va_args {
            HasVaArgs::None => Err(CompileError::NoVarArgsAvailable),
            HasVaArgs::Some => Ok(()),
        }
    }

    /// Get the current offset in the instruction stream.
    pub(crate) fn next_instruction(&self) -> usize {
        self.scope.instructions().len()
    }

    /// Add a label tracking the current instruction position that can be
    /// referenced by labeled jumps.
    pub(crate) fn label_current_instruction(&mut self, label: LabelId) -> Result<(), CompileError> {
        self.scope.add_label(label)
    }

    /// Create a new, unique label for an if statement.
    pub(crate) fn create_if_label(&mut self) -> LabelId {
        self.scope.next_if_id()
    }

    /// Get the current active loop label if the current scope is nested inside
    /// of a loop.
    pub(crate) fn current_loop_label(&self) -> Option<LabelId> {
        self.scope.current_loop_id()
    }

    /// Create a new loop label. The caller must call pop_loop_label after
    /// using.
    pub(crate) fn push_loop_label(&mut self) -> LabelId {
        self.scope.push_loop_id()
    }

    /// Pop the current loop label.
    pub(crate) fn pop_loop_label(&mut self) {
        self.scope.pop_loop_id();
    }

    /// Emit an instruction jumping to a label. If the specified label does not
    /// exist, it will default to raising an error. If the label is added later
    /// in the scope, the instruction will be updated to jump to that location.
    pub(crate) fn emit_jump_label(&mut self, label: LabelId) {
        self.scope.emit_jump_label(label);
    }

    /// Emit a new opcode to the current instruction stream. Returns the
    /// location in the instruction stream.
    pub(crate) fn emit(&mut self, opcode: impl Into<UnasmOp>) -> usize {
        self.scope.emit(opcode)
    }

    /// Overwrite the instruction at location.
    pub(crate) fn overwrite(&mut self, location: usize, opcode: impl Into<UnasmOp>) {
        self.scope.overwrite(location, opcode);
    }

    /// Get the number of locals created in this function so far.
    pub(crate) fn scope_declared_locals_count(&self) -> usize {
        self.scope.total_locals()
    }

    /// Map a new register for a local variable.
    pub(crate) fn new_local(
        &mut self,
        ident: Ident,
    ) -> Result<UninitRegister<MappedLocalRegister>, CompileError> {
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
    ) -> impl ExactSizeIterator<Item = UninitRegister<AnonymousRegister>> + Clone {
        let start = self.scope.total_anons();
        let range = start..(start + size);
        for _ in range.clone() {
            let _ = self.new_anon_reg().no_init_needed();
        }

        range.map(|idx| UninitRegister::from(AnonymousRegister::from(idx)))
    }

    /// Allocate a new anonymous register.
    pub(crate) fn output_to_reg_reuse_anon(&mut self, output: NodeOutput) -> AnonymousRegister {
        match output {
            NodeOutput::Immediate(imm) => imm,
            other => self.new_anon_reg().init_from_node_output(self, other),
        }
    }

    /// Lookup the appropriate register for a specific identifier.
    pub(crate) fn read_variable(
        &mut self,
        ident: Ident,
    ) -> Result<MappedLocalRegister, CompileError> {
        Ok(self.scope.get_in_scope(ident)?.into())
    }

    /// Instruct the compiler to emit a sequence of instruction corresponding to
    /// raising an error with a compile-time known type.
    pub(crate) fn write_raise(&mut self, err: OpError) -> OpError {
        self.emit(opcodes::Raise::from(err));
        err
    }

    /// Emit the instructions for a function.
    pub(crate) fn emit_fn(
        &mut self,
        has_va_args: HasVaArgs,
        params: impl ExactSizeIterator<Item = Ident>,
        body: impl ExactSizeIterator<Item = impl CompileStatement>,
        ret: Option<&impl CompileStatement>,
    ) -> Result<FuncId, CompileError> {
        let mut new_function = self.scope.new_function(params.len());

        {
            let mut new_context = CompilerContext {
                functions: self.functions,
                scope: new_function.start(),
                has_va_args,
            };

            for param in params {
                // TODO(compiler-opt): Technically today this allocates an extra, unused
                // register for every duplicate identifier in the parameter list. It
                // still works fine though, because the number of registers is
                // correct.
                new_context.new_local(param)?.no_init_needed();
            }

            for stat in body {
                stat.compile(&mut new_context)?;
            }

            match ret {
                Some(ret) => ret.compile(&mut new_context)?,
                None => {
                    new_context.emit(opcodes::Op::Ret);
                    None
                }
            };
        }

        let fn_id = self.functions.len();

        self.functions.push(new_function.complete());

        Ok(FuncId::from(fn_id))
    }

    /// Instruct the compiler to create a new context for the current function
    /// in its own subscope and run the provided closure in that scope.
    pub(crate) fn emit_in_subscope(
        &mut self,
        mut emitter: impl FnMut(&mut CompilerContext) -> Result<Option<OpError>, CompileError>,
    ) -> Result<Option<OpError>, CompileError> {
        let scope = self.scope.subscope();

        let mut new_context = CompilerContext {
            functions: self.functions,
            scope,
            has_va_args: self.has_va_args,
        };

        let pending_scope_push = new_context.emit(opcodes::Raise::from(OpError::ByteCodeError {
            err: ByteCodeError::MissingScopeDescriptor,
            offset: new_context.next_instruction(),
        }));

        emitter(&mut new_context)?;

        new_context.overwrite(
            pending_scope_push,
            opcodes::ScopeDescriptor::from(new_context.scope_declared_locals_count()),
        );
        new_context.emit(opcodes::Op::PopScope);

        Ok(None)
    }

    /// Instruct the compiler to emit the instructions required to initialize a
    /// table.
    pub(crate) fn init_table(&mut self) -> AnonymousRegister {
        self.new_anon_reg().init_alloc_table(self)
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
            let retval = outputs
                .next()
                .expect("Still in bounds for outputs")
                .compile(self)?;

            let ret = self.new_anon_reg().init_from_node_output(self, retval);
            self.emit(opcodes::SetRet::from(ret));
        }

        match outputs
            .next()
            .expect("Still in bounds for outputs")
            .compile(self)?
        {
            NodeOutput::ReturnValues => {
                self.emit(opcodes::Op::CopyRetFromRetAndRet);
            }
            NodeOutput::VAStack => {
                self.emit(opcodes::Op::CopyRetFromVaAndRet);
            }
            retval => {
                let ret = self.new_anon_reg().init_from_node_output(self, retval);
                self.emit(opcodes::SetRet::from(ret));
                self.emit(opcodes::Op::Ret);
            }
        }

        debug_assert!(outputs.next().is_none());

        Ok(None)
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
        Op: From<(AnonymousRegister, AnonymousRegister, AnonymousRegister)> + Into<UnasmOp>,
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
            (lhs, rhs) => {
                let lhs = self.new_anon_reg().init_from_node_output(self, lhs);
                let rhs = self.new_anon_reg().init_from_node_output(self, rhs);
                let dst = self.new_anon_reg().no_init_needed();

                self.emit(Op::from((dst, lhs, rhs)));

                Ok(NodeOutput::Immediate(dst))
            }
        }
    }

    pub(crate) fn write_unary_op<Op, Operand, ConstEval>(
        &mut self,
        operand: Operand,
        consteval: ConstEval,
    ) -> Result<NodeOutput, CompileError>
    where
        Op: From<(AnonymousRegister, AnonymousRegister)> + Into<UnasmOp>,
        Operand: CompileExpression,
        ConstEval: FnOnce(Constant) -> Result<Constant, OpError>,
    {
        match operand.compile(self)? {
            NodeOutput::Constant(c) => match consteval(c) {
                Ok(val) => Ok(NodeOutput::Constant(val)),
                Err(err) => Ok(NodeOutput::Err(self.write_raise(err))),
            },
            src => {
                let src = self.new_anon_reg().init_from_node_output(self, src);
                let dst = self.new_anon_reg().no_init_needed();

                self.emit(Op::from((dst, src)));

                Ok(NodeOutput::Immediate(dst))
            }
        }
    }
}
