use std::num::NonZeroUsize;

use derive_more::From;
use either::Either;
use tlua_bytecode::{
    opcodes,
    AnonymousRegister,
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
pub(crate) enum HasVaArgs {
    None,
    Some,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) enum LabelId {}

#[derive(Debug, Default)]
pub(crate) struct Compiler {
    scope: RootScope,
    functions: Vec<UnasmFunction>,
}

impl Compiler {
    pub(crate) fn compile_ast(mut self, ast: Block) -> Result<Chunk, CompileError> {
        let main = self.emit_in_main(|context| ast.compile(context).map(|_| ()))?;

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

pub(crate) trait InitRegister<RegisterTy>: Sized
where
    RegisterTy: Copy,
    UnasmRegister: From<RegisterTy>,
{
    /// Indicate that the register should always init to nil, and needs no
    /// special handling.
    fn no_init_needed(self) -> RegisterTy;

    /// Initialize the register from node output.
    fn init_from_node_output(
        self,
        compiler: &mut CompilerContext,
        value: NodeOutput,
    ) -> Either<RegisterTy, OpError> {
        match value {
            NodeOutput::Constant(value) => Either::Left(self.init_from_const(compiler, value)),
            NodeOutput::Register(source) => Either::Left(self.init_from_reg(compiler, source)),
            NodeOutput::ReturnValues => Either::Left(self.init_from_ret(compiler)),
            NodeOutput::VAStack => Either::Left(self.init_from_va(compiler, 0)),
            NodeOutput::Err(err) => Either::Right(err),
        }
    }

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
    fn no_init_needed(self) -> Self {
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
    pub(crate) fn current_instruction(&self) -> usize {
        self.scope.instructions().len()
    }

    /// Emit a new opcode to the current instruction stream. Returns the
    /// location in the instruction stream.
    pub(crate) fn emit(&mut self, opcode: impl Into<UnasmOp>) -> usize {
        self.scope.emit(opcode);
        self.current_instruction() - 1
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
        let start = self.scope.total_anons();
        let range = start..(start + size);
        for _ in range.clone() {
            let _ = self.new_anon_reg().no_init_needed();
        }

        (
            start,
            range.map(|idx| UninitRegister::from(AnonymousRegister::from(idx))),
        )
    }

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

    /// Emit the instructions for a function.
    pub(crate) fn emit_fn(
        &mut self,
        has_va_args: HasVaArgs,
        params: impl ExactSizeIterator<Item = Ident>,
        body: impl ExactSizeIterator<Item = impl CompileStatement>,
        ret: Option<&impl CompileStatement>,
    ) -> Result<TypeMeta, CompileError> {
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

        Ok(TypeMeta::from(NonZeroUsize::try_from(fn_id + 1).ok()))
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

        emitter(&mut new_context)
    }

    /// Instruct the compiler to emit the instructions required to initialize a
    /// table.
    pub(crate) fn init_table(&mut self) -> UnasmRegister {
        self.new_anon_reg().init_alloc_table(self).into()
    }

    pub(crate) fn emit_store_table(
        &mut self,
        table: UnasmRegister,
        index: UnasmOperand,
        value: NodeOutput,
    ) {
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
                self.emit_store_table(table, index.into(), value);
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
                self.emit_store_table(table, index, value);

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
            self.new_anon_reg()
                .init_from_node_output(self, value)
                .expect_left("Errors should not be handled by storing them in registers.")
        }
    }
}
