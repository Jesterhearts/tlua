use crate::{
    compiling::Chunk,
    vm::{
        binop::{
            f64inbounds,
            traits::ApplyBinop,
        },
        opcodes::{
            Op,
            *,
        },
        runtime::value::{
            function::{
                Scope,
                ScopeSet,
            },
            Function,
            Number,
        },
        ByteCodeError,
        OpError,
        Register,
        Value,
    },
};

pub(crate) struct Context<'call> {
    pub(crate) in_scope: ScopeSet,
    pub(crate) instruction: usize,

    chunk: &'call Chunk,
    instructions: &'call Vec<Instruction>,
}

impl<'call> Context<'call> {
    pub(crate) fn new(scopes: ScopeSet, chunk: &'call Chunk) -> Self {
        Self {
            in_scope: scopes,
            chunk,
            instructions: &chunk.main.instructions,
            instruction: 0,
        }
    }
}

impl Context<'_> {
    fn subcontext<'f, 's>(
        &'s mut self,
        func: &'f Function,
        va_args: Vec<Value>,
        new_scope: Scope,
    ) -> Context<'s>
    where
        'f: 's,
    {
        let func_def = &self.chunk.functions[*func.id];

        Context {
            in_scope: ScopeSet::new(
                func.referenced_scopes.clone(),
                new_scope,
                vec![Value::Nil; func_def.anon_registers],
                va_args,
            ),
            instruction: 0,

            chunk: self.chunk,
            instructions: &func_def.instructions,
        }
    }

    pub(crate) fn execute(mut self) -> Result<Vec<Value>, OpError> {
        while self.instruction < self.instructions.len() {
            let instruction = self.instructions[self.instruction];
            self.instruction += 1;

            match instruction {
                // Numeric operations
                Op::Add(data) => data.apply(&mut self.in_scope)?,
                Op::AddIndirect(data) => data.apply(&mut self.in_scope)?,
                Op::Subtract(data) => data.apply(&mut self.in_scope)?,
                Op::SubtractIndirect(data) => data.apply(&mut self.in_scope)?,
                Op::Times(data) => data.apply(&mut self.in_scope)?,
                Op::TimesIndirect(data) => data.apply(&mut self.in_scope)?,
                Op::Modulo(data) => data.apply(&mut self.in_scope)?,
                Op::ModuloIndirect(data) => data.apply(&mut self.in_scope)?,
                Op::Divide(data) => data.apply(&mut self.in_scope)?,
                Op::DivideIndirect(data) => data.apply(&mut self.in_scope)?,
                Op::Exponetiation(data) => data.apply(&mut self.in_scope)?,
                Op::ExponetiationIndirect(data) => data.apply(&mut self.in_scope)?,
                Op::IDiv(data) => data.apply(&mut self.in_scope)?,
                Op::IDivIndirect(data) => data.apply(&mut self.in_scope)?,
                Op::BitAnd(data) => data.apply(&mut self.in_scope)?,
                Op::BitAndIndirect(data) => data.apply(&mut self.in_scope)?,
                Op::BitOr(data) => data.apply(&mut self.in_scope)?,
                Op::BitOrIndirect(data) => data.apply(&mut self.in_scope)?,
                Op::BitXor(data) => data.apply(&mut self.in_scope)?,
                Op::BitXorIndirect(data) => data.apply(&mut self.in_scope)?,
                Op::ShiftLeft(data) => data.apply(&mut self.in_scope)?,
                Op::ShiftLeftIndirect(data) => data.apply(&mut self.in_scope)?,
                Op::ShiftRight(data) => data.apply(&mut self.in_scope)?,
                Op::ShiftRightIndirect(data) => data.apply(&mut self.in_scope)?,

                // Unary math operations
                Op::UnaryMinus(UnaryMinus { reg }) => {
                    self.in_scope.store(
                        reg,
                        match self.in_scope.load(reg) {
                            Value::Number(operand) => Value::Number(match operand {
                                Number::Float(f) => Number::Float(-f),
                                Number::Integer(i) => Number::Integer(-i),
                            }),
                            _ => todo!(),
                        },
                    );
                }
                Op::UnaryBitNot(UnaryBitNot { reg }) => {
                    self.in_scope.store(
                        reg,
                        Value::Number(Number::Integer(match self.in_scope.load(reg) {
                            Value::Number(operand) => match operand {
                                Number::Float(f) => !f64inbounds(f)?,
                                Number::Integer(i) => !i,
                            },
                            _ => todo!(),
                        })),
                    );
                }

                // Comparison operations
                Op::LessThan(_) => todo!(),
                Op::LessThanIndirect(_) => todo!(),
                Op::LessEqual(_) => todo!(),
                Op::LessEqualIndirect(_) => todo!(),
                Op::GreaterThan(_) => todo!(),
                Op::GreaterThanIndirect(_) => todo!(),
                Op::GreaterEqual(_) => todo!(),
                Op::GreaterEqualIndirect(_) => todo!(),
                Op::Equals(_) => todo!(),
                Op::EqualsIndirect(_) => todo!(),
                Op::NotEqual(_) => todo!(),
                Op::NotEqualIndirect(_) => todo!(),

                // Boolean operations
                Op::And(_) => todo!(),
                Op::AndIndirect(_) => todo!(),
                Op::Or(_) => todo!(),
                Op::OrIndirect(_) => todo!(),

                // Unary boolean operations
                Op::Not(Not { reg }) => {
                    self.in_scope
                        .store(reg, Value::Bool(!self.in_scope.load(reg).as_bool()));
                }

                // String & Array operations
                Op::Concat(_) => todo!(),
                Op::ConcatIndirect(_) => todo!(),
                Op::Length(_) => todo!(),

                Op::Jump(Jump { target }) => {
                    self.instruction = target;
                }

                Op::JumpNot(JumpNot { cond, target }) => {
                    let cond = self.in_scope.load(cond);
                    if !cond.as_bool() {
                        self.instruction = target;
                    }
                }

                Op::JumpNotVa0(JumpNotVa0 { target }) => {
                    if !self.in_scope.load_va(0).as_bool() {
                        self.instruction = target;
                    }
                }

                // Register operations
                Op::Set(Set { dest, source }) => {
                    self.in_scope.store(dest, source.into());
                }
                Op::SetIndirect(SetIndirect { dest, source }) => self.in_scope.copy(dest, source),
                Op::SetFromVa(SetFromVa { dest, index }) => {
                    self.in_scope
                        .store(dest, self.in_scope.load_va(index).clone());
                }

                // Begin calling a function
                Op::StartCall(StartCall { target }) => {
                    self.start_call(target, None)?;
                }

                // Set up return values for a function
                Op::SetRet(SetRet { value }) => {
                    self.in_scope.add_result(value.into());
                }
                Op::SetRetIndirect(SetRetIndirect { src }) => {
                    self.in_scope.add_result(self.in_scope.load(src));
                }
                Op::SetRetVa0 => {
                    self.in_scope.add_result(self.in_scope.load_va(0));
                }

                // Alter the active scopes
                Op::PushScope(descriptor) => {
                    self.in_scope.push_scope(descriptor);
                }

                Op::PopScope => {
                    self.in_scope.pop_scope();
                }

                Op::Ret => {
                    return Ok(self.in_scope.into_results().into());
                }

                Op::CopyRetFromVaAndRet => {
                    let (mut results, va) = self.in_scope.into_results_and_va();
                    results.extend(Vec::from(va).into_iter());
                    return Ok(results.into());
                }

                // Stop execution by raising an error.
                Op::Raise(Raise { err }) => return Err(err),

                Op::StartCallExtending(_)
                | Op::MapArg(_)
                | Op::MapArgIndirect(_)
                | Op::MapVa0
                | Op::DoCall
                | Op::MapVarArgsAndDoCall
                | Op::MapRet(_)
                | Op::SetRetFromRet0
                | Op::JumpNotRet0(_)
                | Op::CopyRetFromRetAndRet => {
                    return Err(OpError::ByteCodeError {
                        err: ByteCodeError::UnexpectedCallInstruction,
                        offset: self.instruction,
                    });
                }
            }
        }

        Ok(self.in_scope.into_results().into())
    }

    // TODO(functions): alloc call isn
    fn start_call(
        &mut self,
        target: Register,
        other_results: Option<Vec<Value>>,
    ) -> Result<(), OpError> {
        let func = match self.in_scope.load(target) {
            Value::Function(ptr) => ptr,
            _ => todo!("Metatables are not supported"),
        };

        let results = self.execute_call(&func.borrow(), other_results)?;

        match self.instructions[self.instruction] {
            // We just performed a call, so if the very next instruction is StartCallExtending, we
            // know that we should include the results in that call directly rather than doing
            // normal result mapping.
            Op::StartCallExtending(StartCallExtending { target }) => {
                self.instruction += 1;
                self.start_call(target, Some(results))
            }
            // We just performed a call, so if the very next instruction is CopyRetFromRet, we know
            // we should copy over all of the results directly rather than doing normal result
            // mapping.
            Op::CopyRetFromRetAndRet => {
                self.in_scope.extend_results(results);

                // Terminate execution by setting the current IP to the end
                self.instruction = self.instructions.len();
                Ok(())
            }
            _ => self.map_results(results),
        }
    }

    fn execute_call(
        &mut self,
        func: &Function,
        other_results: Option<Vec<Value>>,
    ) -> Result<Vec<Value>, OpError> {
        let mut argi = 0;

        let func_def = &self.chunk.functions[*func.id];
        let argc = func_def.named_args;
        let subscope = Scope::new(func_def.local_registers);

        let mut va_args = vec![];

        while self.instruction < self.instructions.len() {
            let isn = self.instructions[self.instruction];
            self.instruction += 1;

            match isn {
                Op::DoCall => {
                    let mut other_results = other_results.into_iter().flat_map(Vec::into_iter);

                    for arg in argi..argc {
                        if let Some(init) = other_results.next() {
                            subscope.registers[arg].replace(init);
                        } else {
                            break;
                        }
                    }

                    va_args.extend(other_results);

                    return self.subcontext(func, va_args, subscope).execute();
                }
                Op::MapVarArgsAndDoCall => {
                    let mut parent_va = self.in_scope.iter_va().cloned();

                    for arg in argi..argc {
                        if let Some(init) = parent_va.next() {
                            subscope.registers[arg].replace(init);
                        } else {
                            break;
                        }
                    }

                    va_args.extend(parent_va);

                    return self.subcontext(func, va_args, subscope).execute();
                }
                Op::MapArg(MapArg { value }) => {
                    if argi < argc {
                        subscope.registers[argi].replace(value.into());
                        argi += 1;
                    } else {
                        va_args.push(value.into());
                    }
                }
                Op::MapArgIndirect(MapArgIndirect { src }) => {
                    if argi < argc {
                        subscope.registers[argi].replace(self.in_scope.load(src));
                        argi += 1;
                    } else {
                        va_args.push(self.in_scope.load(src));
                    }
                }
                Op::MapVa0 => {
                    if argi < argc {
                        subscope.registers[argi].replace(self.in_scope.load_va(0));
                        argi += 1;
                    } else {
                        va_args.push(self.in_scope.load_va(0));
                    }
                }

                _ => {
                    return Err(OpError::ByteCodeError {
                        err: ByteCodeError::ExpectedCallInstruction,
                        offset: self.instruction,
                    })
                }
            }
        }

        Err(OpError::ByteCodeError {
            err: ByteCodeError::MissingCallInvocation,
            offset: self.instruction,
        })
    }

    fn map_results(&mut self, mut results: Vec<Value>) -> Result<(), OpError> {
        let mut results = results.drain(..);

        while self.instruction < self.instructions.len() {
            let isn = self.instructions[self.instruction];

            match isn {
                Op::MapRet(MapRet { dest }) => {
                    self.in_scope
                        .store(dest, results.next().unwrap_or(Value::Nil));
                }

                Op::SetRetFromRet0 => {
                    self.in_scope
                        .add_result(results.next().unwrap_or(Value::Nil));
                }

                Op::StartCallExtending(StartCallExtending { .. }) => {
                    return Err(OpError::ByteCodeError {
                        err: ByteCodeError::UnexpectedCallInstruction,
                        offset: self.instruction,
                    })
                }

                Op::JumpNotRet0(JumpNotRet0 { target }) => {
                    if !results.next().unwrap_or(Value::Nil).as_bool() {
                        self.instruction = target;
                    }
                }

                _ => return Ok(()),
            }

            self.instruction += 1;
        }

        Ok(())
    }
}
