use std::{
    cell::RefCell,
    ops::{
        Index,
        IndexMut,
        Range,
    },
    rc::Rc,
};

use derive_more::{
    Deref,
    DerefMut,
    From,
};
use tlua_bytecode::{
    binop::f64inbounds,
    opcodes::{
        Op,
        *,
    },
    ByteCodeError,
    ImmediateRegister,
    OpError,
    PrimitiveType,
    Truthy,
    TypeId,
};
use tlua_compiler::{
    BuiltinType,
    Chunk,
};
use tracing_rc::rc::Gc;

use crate::vm::{
    binop::{
        bool_op,
        cmp_op,
        concat_op,
        fp_op,
        int_op,
    },
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

#[derive(Debug, Deref, DerefMut, From)]
pub(crate) struct Immediates(Vec<Value>);

impl Index<ImmediateRegister> for Immediates {
    type Output = Value;

    fn index(&self, index: ImmediateRegister) -> &Self::Output {
        &self.0[usize::from(index)]
    }
}

impl IndexMut<ImmediateRegister> for Immediates {
    fn index_mut(&mut self, index: ImmediateRegister) -> &mut Self::Output {
        &mut self.0[usize::from(index)]
    }
}

#[derive(Debug)]
pub struct Context<'call> {
    in_scope: ScopeSet,
    imm: Immediates,

    chunk: &'call Chunk,
    instructions: &'call [Instruction],
    instruction_pointer: &'call [Instruction],
}

impl<'call> Context<'call> {
    pub fn new(scopes: ScopeSet, chunk: &'call Chunk) -> Self {
        Self {
            in_scope: scopes,
            imm: vec![Value::Nil; chunk.main.immediates].into(),
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
            in_scope: ScopeSet::new(func.referenced_scopes.clone(), new_scope, va_args),
            imm: vec![Value::Nil; func_def.immediates].into(),

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
                Op::Add(Add { lhs, rhs }) => {
                    self.imm[lhs] = fp_op::<Add>(lhs, rhs, &self.imm)?;
                }
                Op::Subtract(Subtract { lhs, rhs }) => {
                    self.imm[lhs] = fp_op::<Subtract>(lhs, rhs, &self.imm)?;
                }
                Op::Times(Times { lhs, rhs }) => {
                    self.imm[lhs] = fp_op::<Times>(lhs, rhs, &self.imm)?;
                }
                Op::Modulo(Modulo { lhs, rhs }) => {
                    self.imm[lhs] = fp_op::<Modulo>(lhs, rhs, &self.imm)?;
                }
                Op::Divide(Divide { lhs, rhs }) => {
                    self.imm[lhs] = fp_op::<Divide>(lhs, rhs, &self.imm)?;
                }
                Op::Exponetiation(Exponetiation { lhs, rhs }) => {
                    self.imm[lhs] = fp_op::<Exponetiation>(lhs, rhs, &self.imm)?;
                }
                Op::IDiv(IDiv { lhs, rhs }) => {
                    self.imm[lhs] = fp_op::<IDiv>(lhs, rhs, &self.imm)?;
                }
                Op::BitAnd(BitAnd { lhs, rhs }) => {
                    self.imm[lhs] = int_op::<BitAnd>(lhs, rhs, &self.imm)?;
                }
                Op::BitOr(BitOr { lhs, rhs }) => {
                    self.imm[lhs] = int_op::<BitOr>(lhs, rhs, &self.imm)?;
                }
                Op::BitXor(BitXor { lhs, rhs }) => {
                    self.imm[lhs] = int_op::<BitXor>(lhs, rhs, &self.imm)?;
                }
                Op::ShiftLeft(ShiftLeft { lhs, rhs }) => {
                    self.imm[lhs] = int_op::<ShiftLeft>(lhs, rhs, &self.imm)?;
                }
                Op::ShiftRight(ShiftRight { lhs, rhs }) => {
                    self.imm[lhs] = int_op::<ShiftRight>(lhs, rhs, &self.imm)?;
                }

                // Unary math operations
                Op::UnaryMinus(UnaryMinus { dst, src }) => {
                    self.imm[dst] = match self.imm[src].clone() {
                        Value::Number(operand) => Value::Number(match operand {
                            Number::Float(f) => Number::Float(-f),
                            Number::Integer(i) => Number::Integer(-i),
                        }),
                        _ => return Err(OpError::InvalidType { op: "unary minus" }),
                    };
                }
                Op::UnaryBitNot(UnaryBitNot { dst, src }) => {
                    self.imm[dst] = match self.imm[src].clone() {
                        Value::Number(operand) => Value::Number(match operand {
                            Number::Float(f) => {
                                if f.fract() == 0.0 {
                                    Number::Integer(!f64inbounds(f)?)
                                } else {
                                    return Err(OpError::FloatToIntConversionFailed { f });
                                }
                            }
                            Number::Integer(i) => Number::Integer(!i),
                        }),
                        _ => {
                            return Err(OpError::InvalidType {
                                op: "unary bit not",
                            })
                        }
                    };
                }

                // Comparison operations
                Op::LessThan(LessThan { lhs, rhs }) => {
                    self.imm[lhs] = cmp_op::<LessThan>(lhs, rhs, &self.imm)?;
                }
                Op::LessEqual(LessEqual { lhs, rhs }) => {
                    self.imm[lhs] = cmp_op::<LessEqual>(lhs, rhs, &self.imm)?;
                }
                Op::GreaterThan(GreaterThan { lhs, rhs }) => {
                    self.imm[lhs] = cmp_op::<GreaterThan>(lhs, rhs, &self.imm)?;
                }
                Op::GreaterEqual(GreaterEqual { lhs, rhs }) => {
                    self.imm[lhs] = cmp_op::<GreaterEqual>(lhs, rhs, &self.imm)?;
                }
                Op::Equals(Equals { lhs, rhs }) => {
                    self.imm[lhs] = cmp_op::<Equals>(lhs, rhs, &self.imm)?;
                }
                Op::NotEqual(NotEqual { lhs, rhs }) => {
                    self.imm[lhs] = cmp_op::<NotEqual>(lhs, rhs, &self.imm)?;
                }

                // Boolean operations
                Op::And(And { lhs, rhs }) => {
                    self.imm[lhs] = bool_op::<And>(lhs, rhs, &self.imm);
                }
                Op::Or(Or { lhs, rhs }) => {
                    self.imm[lhs] = bool_op::<Or>(lhs, rhs, &self.imm);
                }

                // Unary boolean operations
                Op::Not(Not { dst, src }) => {
                    self.imm[dst] = Value::Bool(!self.imm[src].as_bool());
                }

                // String & Array operations
                Op::Concat(Concat { lhs, rhs }) => {
                    self.imm[lhs] = concat_op(lhs, rhs, &self.imm)?;
                }
                Op::Length(Length { dst, src }) => {
                    self.imm[dst] = match &self.imm[src] {
                        Value::String(s) => i64::try_from(s.borrow().len())
                            .map_err(|_| OpError::StringLengthOutOfBounds)
                            .map(Value::from)?,
                        _ => return Err(OpError::InvalidType { op: "length" }),
                    };
                }

                Op::Jump(Jump { target }) => {
                    self.instruction_pointer = self.instructions.split_at(target).1;
                }

                Op::JumpNot(JumpNot { cond, target }) => {
                    if !self.imm[cond].as_bool() {
                        self.instruction_pointer = self.instructions.split_at(target).1;
                    }
                }

                Op::JumpNil(JumpNil { cond, target }) => {
                    if self.imm[cond] == Value::Nil {
                        self.instruction_pointer = self.instructions.split_at(target).1;
                    }
                }

                // Table operations
                Op::Lookup(Lookup { dst, src, idx }) => {
                    self.imm[dst] = match &self.imm[src] {
                        Value::Table(t) => t
                            .borrow()
                            .entries
                            .get(&TableKey::try_from(self.imm[idx].clone())?)
                            .cloned()
                            .unwrap_or_default(),
                        _ => return Err(OpError::InvalidType { op: "index" }),
                    };
                }

                Op::SetProperty(SetProperty { dst, idx, src }) => {
                    match &self.imm[dst] {
                        Value::Table(t) => t.borrow_mut().entries.insert(
                            TableKey::try_from(self.imm[idx].clone())?,
                            self.imm[src].clone(),
                        ),
                        _ => return Err(OpError::InvalidType { op: "newindex" }),
                    };
                }

                Op::SetAllPropertiesFromVa(SetAllPropertiesFromVa { dst, start_idx }) => {
                    let entries = self
                        .in_scope
                        .iter_va()
                        .enumerate()
                        .map(|(index, v)| {
                            i64::try_from(index + start_idx)
                                .map_err(|_| OpError::TableIndexOutOfBounds)
                                .map(Value::from)
                                .and_then(TableKey::try_from)
                                .map(|key| (key, v.clone()))
                        })
                        .collect::<Result<Vec<_>, _>>()?;

                    match &self.imm[dst] {
                        Value::Table(t) => t.borrow_mut().entries.extend(entries),
                        _ => return Err(OpError::InvalidType { op: "va tableinit" }),
                    };
                }

                // Register operations
                Op::LoadConstant(LoadConstant { dst, src }) => {
                    self.imm[dst] = match src {
                        Constant::Nil => Value::Nil,
                        Constant::Bool(b) => Value::Bool(b),
                        Constant::Float(f) => Value::Number(Number::Float(f)),
                        Constant::Integer(i) => Value::Number(Number::Integer(i)),
                        Constant::String(s) => Value::String(Rc::new(RefCell::new(
                            self.chunk
                                .strings
                                .get_string(s)
                                .expect("Valid string id")
                                .clone(),
                        ))),
                    };
                }
                Op::LoadVa(LoadVa {
                    dst_start,
                    va_start,
                    count,
                }) => {
                    let mut va = self.in_scope.iter_va().skip(va_start);
                    for dst in self.imm.iter_mut().skip(dst_start).take(count) {
                        *dst = va.next().cloned().unwrap_or_default();
                    }
                }
                Op::LoadRegister(LoadRegister { dst, src }) => {
                    self.imm[dst] = self.in_scope.load(src);
                }
                Op::DuplicateRegister(DuplicateRegister { dst, src }) => {
                    self.imm[dst] = self.imm[src].clone();
                }
                Op::Store(Store { dst, src }) => {
                    self.in_scope.store(dst, self.imm[src].clone());
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
                    self.in_scope.add_result(self.imm[src].clone());
                }

                // Allocate values
                Op::Alloc(Alloc { dst, type_id }) => {
                    self.imm[dst] = match BuiltinType::try_from(type_id) {
                        Ok(BuiltinType::Function(id)) => {
                            Value::Function(Gc::new(Function::new(&self.in_scope, id)))
                        }
                        Ok(BuiltinType::Table) => Value::Table(Gc::new(Table::default())),
                        _ => {
                            return Err(OpError::ByteCodeError {
                                err: ByteCodeError::InvalidTypeId,
                                offset: self.ip_index(),
                            })
                        }
                    };
                }

                Op::CheckType(CheckType {
                    dst,
                    src,
                    expected_type_id,
                }) => {
                    self.imm[dst] = match (expected_type_id, &self.imm[src]) {
                        (TypeId::Primitive(PrimitiveType::Nil), Value::Nil)
                        | (TypeId::Primitive(PrimitiveType::Bool), Value::Bool(_))
                        | (
                            TypeId::Primitive(PrimitiveType::Float),
                            Value::Number(Number::Float(_)),
                        )
                        | (
                            TypeId::Primitive(PrimitiveType::Integer),
                            Value::Number(Number::Integer(_)),
                        )
                        | (TypeId::Primitive(PrimitiveType::String), Value::String(_)) => true,
                        (id @ TypeId::Any(_), target) => {
                            match (BuiltinType::try_from(id), target) {
                                (Ok(BuiltinType::Table), Value::Table(_)) => true,
                                (Ok(BuiltinType::Function(id)), Value::Function(f)) => {
                                    f.borrow().id == id
                                }
                                (_, _) => false,
                            }
                        }
                        _ => false,
                    }
                    .into();
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
                Op::RaiseIfNot(RaiseIfNot { src, err }) => {
                    if !self.imm[src].as_bool() {
                        return Err(err);
                    }
                }

                Op::CallCopyRet(_)
                | Op::ConsumeRetRange(_)
                | Op::SetAllPropertiesFromRet(_)
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
        target: ImmediateRegister,
        arg_range: Range<usize>,
        extra_args: Vec<Value>,
    ) -> Result<(), OpError> {
        let func = match &self.imm[target] {
            Value::Function(ptr) => ptr.clone(),
            _ => return Err(OpError::InvalidType { op: "call" }),
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
            subscope.registers[target_idx].replace(self.imm[src_idx.into()].clone());
        }

        if arg_range.len() < desired_input_args {
            for target_idx in arg_range.len()..desired_input_args {
                subscope.registers[target_idx].replace(other_results.next().unwrap_or_default());
            }
        } else {
            for src_idx in (arg_range.start + desired_input_args)..arg_range.end {
                va_args.push(self.imm[src_idx.into()].clone());
            }
        }

        va_args.extend(other_results);
        self.subcontext(func, va_args, subscope).execute()
    }

    fn map_results(&mut self, results: Vec<Value>) -> Result<(), OpError> {
        let (&isn, next) = if let Some(next) = self.instruction_pointer.split_first() {
            next
        } else {
            return Ok(());
        };

        match isn {
            Op::ConsumeRetRange(ConsumeRetRange { dst_start, count }) => {
                let mut results = results.into_iter();
                for dst in self.imm.iter_mut().skip(dst_start).take(count) {
                    *dst = results.next().unwrap_or_default();
                }
            }

            Op::SetAllPropertiesFromRet(SetAllPropertiesFromRet { dst, start_idx }) => {
                match &self.imm[dst] {
                    Value::Table(t) => {
                        let mut table = t.borrow_mut();
                        for res in results.into_iter().enumerate().map(|(index, v)| {
                            i64::try_from(index + start_idx)
                                .map_err(|_| OpError::TableIndexOutOfBounds)
                                .map(Value::from)
                                .and_then(TableKey::try_from)
                                .map(|key| (key, v))
                        }) {
                            let (k, v) = res?;
                            table.entries.insert(k, v);
                        }
                    }
                    _ => {
                        return Err(OpError::InvalidType {
                            op: "ret table init",
                        })
                    }
                };
            }

            _ => return Ok(()),
        }

        self.instruction_pointer = next;

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
