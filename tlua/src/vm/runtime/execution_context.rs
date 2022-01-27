use std::ops::Range;

use tlua_bytecode::{
    binop::f64inbounds,
    opcodes::{
        Op,
        *,
    },
    ByteCodeError,
    OpError,
    Register,
    Truthy,
};
use tlua_compiler::{
    Chunk,
    TypeIds,
};
use tracing_rc::rc::Gc;

use crate::vm::{
    binop::traits::ApplyBinop,
    runtime::{
        value::{
            function::{
                Scope,
                ScopeSet,
            },
            table::TableKey,
            Function,
            Number,
        },
        Table,
        Value,
    },
};

pub struct Context<'call> {
    in_scope: ScopeSet,

    chunk: &'call Chunk,
    instructions: &'call [Instruction],
    instruction_pointer: &'call [Instruction],
}

impl<'call> Context<'call> {
    pub fn new(scopes: ScopeSet, chunk: &'call Chunk) -> Self {
        Self {
            in_scope: scopes,
            chunk,
            instructions: chunk.main.instructions.as_slice(),
            instruction_pointer: chunk.main.instructions.as_slice(),
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
        let func_def = &self.chunk.functions[usize::from(func.id)];

        Context {
            in_scope: ScopeSet::new(
                func.referenced_scopes.clone(),
                new_scope,
                vec![Value::Nil; func_def.anon_registers],
                va_args,
            ),

            chunk: self.chunk,
            instructions: func_def.instructions.as_slice(),
            instruction_pointer: func_def.instructions.as_slice(),
        }
    }

    pub fn execute(mut self) -> Result<Vec<Value>, OpError> {
        while let Some((&instruction, next)) = self.instruction_pointer.split_first() {
            self.instruction_pointer = next;

            match instruction {
                Op::Nop => (),

                // Numeric operations
                Op::Add(data) => data.apply(&mut self.in_scope)?,
                Op::Subtract(data) => data.apply(&mut self.in_scope)?,
                Op::Times(data) => data.apply(&mut self.in_scope)?,
                Op::Modulo(data) => data.apply(&mut self.in_scope)?,
                Op::Divide(data) => data.apply(&mut self.in_scope)?,
                Op::Exponetiation(data) => data.apply(&mut self.in_scope)?,
                Op::IDiv(data) => data.apply(&mut self.in_scope)?,
                Op::BitAnd(data) => data.apply(&mut self.in_scope)?,
                Op::BitOr(data) => data.apply(&mut self.in_scope)?,
                Op::BitXor(data) => data.apply(&mut self.in_scope)?,
                Op::ShiftLeft(data) => data.apply(&mut self.in_scope)?,
                Op::ShiftRight(data) => data.apply(&mut self.in_scope)?,

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
                                Number::Float(f) => {
                                    if f.fract() == 0.0 {
                                        !f64inbounds(f)?
                                    } else {
                                        return Err(OpError::FloatToIntConversionFailed { f });
                                    }
                                }
                                Number::Integer(i) => !i,
                            },
                            _ => todo!(),
                        })),
                    );
                }

                // Comparison operations
                Op::LessThan(data) => data.apply(&mut self.in_scope)?,
                Op::LessEqual(data) => data.apply(&mut self.in_scope)?,
                Op::GreaterThan(data) => data.apply(&mut self.in_scope)?,
                Op::GreaterEqual(data) => data.apply(&mut self.in_scope)?,
                Op::Equals(data) => data.apply(&mut self.in_scope)?,
                Op::NotEqual(data) => data.apply(&mut self.in_scope)?,

                // Boolean operations
                Op::And(data) => data.apply(&mut self.in_scope)?,
                Op::Or(data) => data.apply(&mut self.in_scope)?,

                // Unary boolean operations
                Op::Not(Not { reg }) => {
                    self.in_scope
                        .store(reg, Value::Bool(!self.in_scope.load(reg).as_bool()));
                }

                // String & Array operations
                Op::Concat(_) => todo!(),
                Op::Length(_) => todo!(),

                Op::Jump(Jump { target }) => {
                    self.instruction_pointer = self.instructions.split_at(target).1;
                }

                Op::JumpNot(JumpNot { cond, target }) => {
                    let cond = self.in_scope.load(cond);
                    if !cond.as_bool() {
                        self.instruction_pointer = self.instructions.split_at(target).1;
                    }
                }

                Op::JumpNotVa0(JumpNotVa0 { target }) => {
                    if !self.in_scope.load_va(0).as_bool() {
                        self.instruction_pointer = self.instructions.split_at(target).1;
                    }
                }

                // Table operations
                Op::Lookup(Lookup { dest, index }) => {
                    let value = match self.in_scope.load(dest) {
                        Value::Table(t) => t
                            .borrow()
                            .entries
                            .get(&TryFrom::<Value>::try_from(
                                Value::try_from(index)
                                    .unwrap_or_else(|reg| self.in_scope.load(reg)),
                            )?)
                            .cloned()
                            .unwrap_or_default(),
                        _ => todo!("metatables are unsupported"),
                    };

                    self.in_scope.store(dest, value);
                }

                // TODO(cleanup): These can have generic behavior across their arguments.
                Op::Store(Store { dest, src, index }) => {
                    let value = Value::try_from(src).unwrap_or_else(|reg| self.in_scope.load(reg));
                    match self.in_scope.load(dest) {
                        Value::Table(t) => t.borrow_mut().entries.insert(
                            TryFrom::<Value>::try_from(
                                Value::try_from(index)
                                    .unwrap_or_else(|reg| self.in_scope.load(reg)),
                            )?,
                            value,
                        ),
                        _ => todo!("metatables are unsupported"),
                    };
                }
                Op::StoreFromVa(StoreFromVa {
                    dest,
                    index,
                    va_index,
                }) => {
                    match self.in_scope.load(dest) {
                        Value::Table(t) => t.borrow_mut().entries.insert(
                            TryFrom::<Value>::try_from(
                                Value::try_from(index)
                                    .unwrap_or_else(|reg| self.in_scope.load(reg)),
                            )?,
                            self.in_scope.load_va(va_index),
                        ),
                        _ => todo!("metatables are unsupported"),
                    };
                }

                Op::StoreAllFromVa(StoreAllFromVa { dest, start_index }) => {
                    let entries = self
                        .in_scope
                        .iter_va()
                        .enumerate()
                        .map(|(index, v)| {
                            i64::try_from(index + start_index)
                                .map_err(|_| OpError::TableIndexOutOfBounds)
                                .map(Value::from)
                                .and_then(TableKey::try_from)
                                .map(|key| (key, v.clone()))
                        })
                        .collect::<Result<Vec<_>, _>>()?;

                    match self.in_scope.load(dest) {
                        Value::Table(t) => t.borrow_mut().entries.extend(entries),
                        _ => todo!("metatables are unsupported"),
                    };
                }

                // Register operations
                Op::Set(Set { dest, source }) => {
                    self.in_scope.store(
                        dest,
                        Value::try_from(source).unwrap_or_else(|reg| self.in_scope.load(reg)),
                    );
                }
                Op::SetFromVa(SetFromVa { dest, index }) => {
                    self.in_scope
                        .store(dest, self.in_scope.load_va(index).clone());
                }

                // Begin calling a function
                Op::Call(Call {
                    target,
                    mapped_args_start,
                    mapped_args_count,
                }) => {
                    self.start_call(
                        target,
                        mapped_args_start..(mapped_args_start + mapped_args_count),
                        Vec::default(),
                    )?;
                }
                Op::CallCopyVa(CallCopyVa {
                    target,
                    mapped_args_start,
                    mapped_args_count,
                }) => {
                    self.start_call(
                        target,
                        mapped_args_start..(mapped_args_start + mapped_args_count),
                        self.in_scope.iter_va().cloned().collect(),
                    )?;
                }

                // Set up return values for a function
                Op::SetRet(SetRet { src }) => {
                    self.in_scope.add_result(
                        Value::try_from(src).unwrap_or_else(|reg| self.in_scope.load(reg)),
                    );
                }
                Op::SetRetVa0 => {
                    self.in_scope.add_result(self.in_scope.load_va(0));
                }

                // Allocate values
                Op::Alloc(Alloc {
                    dest,
                    type_id,
                    metadata,
                }) => {
                    let value = match type_id {
                        TypeIds::FUNCTION => Value::Function(Gc::new(Function::new(
                            &self.in_scope,
                            metadata.try_into().map_err(|err| OpError::ByteCodeError {
                                err,
                                offset: self.ip_index(),
                            })?,
                        ))),
                        TypeIds::TABLE => Value::Table(Gc::new(Table::default())),
                        _ => {
                            return Err(OpError::ByteCodeError {
                                err: ByteCodeError::InvalidTypeId,
                                offset: self.ip_index(),
                            })
                        }
                    };
                    self.in_scope.store(dest, value);
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

                Op::CallCopyRet(_)
                | Op::MapRet(_)
                | Op::StoreRet(_)
                | Op::StoreAllRet(_)
                | Op::SetRetFromRet0
                | Op::JumpNotRet0(_)
                | Op::CopyRetFromRetAndRet => {
                    return Err(OpError::ByteCodeError {
                        err: ByteCodeError::UnexpectedCallInstruction,
                        offset: self.ip_index(),
                    });
                }
            }
        }

        Ok(self.in_scope.into_results().into())
    }

    fn start_call(
        &mut self,
        target: AnyReg<Register>,
        arg_range: Range<usize>,
        extra_args: Vec<Value>,
    ) -> Result<(), OpError> {
        let func = match self.in_scope.load(target) {
            Value::Function(ptr) => ptr,
            _ => todo!("Metatables are not supported"),
        };

        let results = self.execute_call(&func.borrow(), arg_range, extra_args)?;

        match self.instruction_pointer[0] {
            // We just performed a call, so if the very next instruction is StartCallExtending, we
            // know that we should include the results in that call directly rather than doing
            // normal result mapping.
            Op::CallCopyRet(CallCopyRet {
                target,
                mapped_args_start,
                mapped_args_count,
            }) => {
                self.instruction_pointer = self
                    .instruction_pointer
                    .split_first()
                    .map(|(_, next)| next)
                    .unwrap_or_default();

                self.start_call(
                    target,
                    mapped_args_start..(mapped_args_start + mapped_args_count),
                    results,
                )
            }
            // We just performed a call, so if the very next instruction is CopyRetFromRet, we know
            // we should copy over all of the results directly rather than doing normal result
            // mapping.
            Op::CopyRetFromRetAndRet => {
                self.in_scope.extend_results(results);
                self.instruction_pointer = &[];

                Ok(())
            }
            _ => self.map_results(results),
        }
    }

    fn execute_call(
        &mut self,
        func: &Function,
        arg_range: Range<usize>,
        mut extra_args: Vec<Value>,
    ) -> Result<Vec<Value>, OpError> {
        let func_def = &self.chunk.functions[usize::from(func.id)];
        let desired_input_args = func_def.named_args;
        let subscope = Scope::new(func_def.local_registers);

        let mut other_results = extra_args.drain(..);

        let total_input_args = arg_range.len() + other_results.len();
        let mut va_args = if total_input_args > desired_input_args {
            Vec::with_capacity(total_input_args - desired_input_args)
        } else {
            Vec::default()
        };

        // Map all of the explicit input args to target registers
        for (target_idx, src_idx) in (0..desired_input_args).zip(arg_range.clone()) {
            subscope.registers[target_idx].replace(self.in_scope.load_anon_offset(src_idx));
        }

        if arg_range.len() < desired_input_args {
            for target_idx in arg_range.len()..desired_input_args {
                subscope.registers[target_idx].replace(other_results.next().unwrap_or_default());
            }
        } else {
            for src_idx in (arg_range.start + desired_input_args)..arg_range.end {
                va_args.push(self.in_scope.load_anon_offset(src_idx));
            }
        }

        va_args.extend(other_results);
        self.subcontext(func, va_args, subscope).execute()
    }

    fn map_results(&mut self, mut results: Vec<Value>) -> Result<(), OpError> {
        let mut results = results.drain(..);
        while let Some((&isn, next)) = self.instruction_pointer.split_first() {
            match isn {
                Op::MapRet(MapRet { dest }) => {
                    self.in_scope
                        .store(dest, results.next().unwrap_or(Value::Nil));
                }

                Op::StoreRet(StoreRet { dest, index }) => {
                    match self.in_scope.load(dest) {
                        Value::Table(t) => t.borrow_mut().entries.insert(
                            TryFrom::<Value>::try_from(
                                Value::try_from(index)
                                    .unwrap_or_else(|reg| self.in_scope.load(reg)),
                            )?,
                            results.next().unwrap_or(Value::Nil),
                        ),
                        _ => todo!("metatables are unsupported"),
                    };
                }

                Op::StoreAllRet(StoreAllRet { dest, start_index }) => {
                    match self.in_scope.load(dest) {
                        Value::Table(t) => {
                            let mut table = t.borrow_mut();
                            for res in results.enumerate().map(|(index, v)| {
                                i64::try_from(index + start_index)
                                    .map_err(|_| OpError::TableIndexOutOfBounds)
                                    .map(Value::from)
                                    .and_then(TableKey::try_from)
                                    .map(|key| (key, v))
                            }) {
                                let (k, v) = res?;
                                table.entries.insert(k, v);
                            }
                        }
                        _ => todo!("metatables are unsupported"),
                    };

                    self.instruction_pointer = next;
                    return Ok(());
                }

                Op::SetRetFromRet0 => {
                    self.in_scope
                        .add_result(results.next().unwrap_or(Value::Nil));
                }

                Op::JumpNotRet0(JumpNotRet0 { target }) => {
                    if !results.next().unwrap_or(Value::Nil).as_bool() {
                        self.instruction_pointer = self.instructions.split_at(target).1;
                    }
                }

                _ => return Ok(()),
            }

            self.instruction_pointer = next;
        }

        Ok(())
    }

    fn ip_index(&self) -> usize {
        if self.instruction_pointer.is_empty() {
            self.instructions.len()
        } else {
            (self.instruction_pointer.as_ptr() as usize - self.instructions.as_ptr() as usize)
                / std::mem::size_of::<Instruction>()
        }
    }
}
