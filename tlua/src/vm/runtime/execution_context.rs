use std::ops::{
    Index,
    IndexMut,
    Range,
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
    AnonymousRegister,
    ByteCodeError,
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

impl Index<AnonymousRegister> for Immediates {
    type Output = Value;

    fn index(&self, index: AnonymousRegister) -> &Self::Output {
        &self.0[usize::from(index)]
    }
}

impl IndexMut<AnonymousRegister> for Immediates {
    fn index_mut(&mut self, index: AnonymousRegister) -> &mut Self::Output {
        &mut self.0[usize::from(index)]
    }
}

#[derive(Debug)]
pub struct Context<'call> {
    in_scope: ScopeSet,
    anon: Immediates,

    chunk: &'call Chunk,
    instructions: &'call [Instruction],
    instruction_pointer: &'call [Instruction],
}

impl<'call> Context<'call> {
    pub fn new(scopes: ScopeSet, chunk: &'call Chunk) -> Self {
        Self {
            in_scope: scopes,
            anon: vec![Value::Nil; chunk.main.anon_registers].into(),
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
            anon: vec![Value::Nil; func_def.anon_registers].into(),

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
                Op::Add(Add { dst, lhs, rhs }) => {
                    self.anon[dst] = fp_op::<Add>(lhs, rhs, &self.anon)?;
                }
                Op::Subtract(Subtract { dst, lhs, rhs }) => {
                    self.anon[dst] = fp_op::<Subtract>(lhs, rhs, &self.anon)?;
                }
                Op::Times(Times { dst, lhs, rhs }) => {
                    self.anon[dst] = fp_op::<Times>(lhs, rhs, &self.anon)?;
                }
                Op::Modulo(Modulo { dst, lhs, rhs }) => {
                    self.anon[dst] = fp_op::<Modulo>(lhs, rhs, &self.anon)?;
                }
                Op::Divide(Divide { dst, lhs, rhs }) => {
                    self.anon[dst] = fp_op::<Divide>(lhs, rhs, &self.anon)?;
                }
                Op::Exponetiation(Exponetiation { dst, lhs, rhs }) => {
                    self.anon[dst] = fp_op::<Exponetiation>(lhs, rhs, &self.anon)?;
                }
                Op::IDiv(IDiv { dst, lhs, rhs }) => {
                    self.anon[dst] = fp_op::<IDiv>(lhs, rhs, &self.anon)?;
                }
                Op::BitAnd(BitAnd { dst, lhs, rhs }) => {
                    self.anon[dst] = int_op::<BitAnd>(lhs, rhs, &self.anon)?;
                }
                Op::BitOr(BitOr { dst, lhs, rhs }) => {
                    self.anon[dst] = int_op::<BitOr>(lhs, rhs, &self.anon)?;
                }
                Op::BitXor(BitXor { dst, lhs, rhs }) => {
                    self.anon[dst] = int_op::<BitXor>(lhs, rhs, &self.anon)?;
                }
                Op::ShiftLeft(ShiftLeft { dst, lhs, rhs }) => {
                    self.anon[dst] = int_op::<ShiftLeft>(lhs, rhs, &self.anon)?;
                }
                Op::ShiftRight(ShiftRight { dst, lhs, rhs }) => {
                    self.anon[dst] = int_op::<ShiftRight>(lhs, rhs, &self.anon)?;
                }

                // Unary math operations
                Op::UnaryMinus(UnaryMinus { dst, src }) => {
                    self.anon[dst] = match self.anon[src].clone() {
                        Value::Number(operand) => Value::Number(match operand {
                            Number::Float(f) => Number::Float(-f),
                            Number::Integer(i) => Number::Integer(-i),
                        }),
                        _ => todo!(),
                    };
                }
                Op::UnaryBitNot(UnaryBitNot { dst, src }) => {
                    self.anon[dst] = match self.anon[src].clone() {
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
                        _ => todo!(),
                    };
                }

                // Comparison operations
                Op::LessThan(LessThan { dst, lhs, rhs }) => {
                    self.anon[dst] = cmp_op::<LessThan>(lhs, rhs, &self.anon)?;
                }
                Op::LessEqual(LessEqual { dst, lhs, rhs }) => {
                    self.anon[dst] = cmp_op::<LessEqual>(lhs, rhs, &self.anon)?;
                }
                Op::GreaterThan(GreaterThan { dst, lhs, rhs }) => {
                    self.anon[dst] = cmp_op::<GreaterThan>(lhs, rhs, &self.anon)?;
                }
                Op::GreaterEqual(GreaterEqual { dst, lhs, rhs }) => {
                    self.anon[dst] = cmp_op::<GreaterEqual>(lhs, rhs, &self.anon)?;
                }
                Op::Equals(Equals { dst, lhs, rhs }) => {
                    self.anon[dst] = cmp_op::<Equals>(lhs, rhs, &self.anon)?;
                }
                Op::NotEqual(NotEqual { dst, lhs, rhs }) => {
                    self.anon[dst] = cmp_op::<NotEqual>(lhs, rhs, &self.anon)?;
                }

                // Boolean operations
                Op::And(And { dst, lhs, rhs }) => {
                    self.anon[dst] = bool_op::<And>(lhs, rhs, &self.anon);
                }
                Op::Or(Or { dst, lhs, rhs }) => {
                    self.anon[dst] = bool_op::<Or>(lhs, rhs, &self.anon);
                }

                // Unary boolean operations
                Op::Not(Not { dst, src }) => {
                    self.anon[dst] = Value::Bool(!self.anon[src].as_bool());
                }

                // String & Array operations
                Op::Concat(_) => todo!(),
                Op::Length(Length { dst, src }) => {
                    self.anon[dst] = match &self.anon[src] {
                        Value::String(s) => i64::try_from(s.borrow().len())
                            .map_err(|_| OpError::StringLengthOutOfBounds)
                            .map(Value::from)?,
                        Value::Table(_) => todo!(),
                        _ => return Err(OpError::InvalidType { op: "length" }),
                    };
                }

                Op::Jump(Jump { target }) => {
                    self.instruction_pointer = self.instructions.split_at(target).1;
                }

                Op::JumpNot(JumpNot { cond, target }) => {
                    if !self.anon[cond].as_bool() {
                        self.instruction_pointer = self.instructions.split_at(target).1;
                    }
                }

                Op::JumpNil(JumpNil { cond, target }) => {
                    if self.anon[cond] == Value::Nil {
                        self.instruction_pointer = self.instructions.split_at(target).1;
                    }
                }

                // Table operations
                Op::Lookup(Lookup { dst, src, idx }) => {
                    self.anon[dst] = match &self.anon[src] {
                        Value::Table(t) => t
                            .borrow()
                            .entries
                            .get(&TableKey::try_from(self.anon[idx].clone())?)
                            .cloned()
                            .unwrap_or_default(),
                        _ => todo!("metatables are unsupported"),
                    };
                }

                Op::SetProperty(SetProperty { dst, idx, src }) => {
                    match &self.anon[dst] {
                        Value::Table(t) => t.borrow_mut().entries.insert(
                            TableKey::try_from(self.anon[idx].clone())?,
                            self.anon[src].clone(),
                        ),
                        _ => todo!("metatables are unsupported"),
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

                    match &self.anon[dst] {
                        Value::Table(t) => t.borrow_mut().entries.extend(entries),
                        _ => todo!("metatables are unsupported"),
                    };
                }

                // Register operations
                Op::LoadConstant(LoadConstant { dst, src }) => {
                    self.anon[dst] = src.into();
                }
                Op::LoadVa(LoadVa {
                    dst_start,
                    va_start,
                    count,
                }) => {
                    let mut va = self.in_scope.iter_va().skip(va_start);
                    for dst in self.anon.iter_mut().skip(dst_start).take(count) {
                        *dst = va.next().cloned().unwrap_or_default();
                    }
                }
                Op::LoadRegister(LoadRegister { dst, src }) => {
                    self.anon[dst] = self.in_scope.load(src);
                }
                Op::DuplicateRegister(DuplicateRegister { dst, src }) => {
                    self.anon[dst] = self.anon[src].clone();
                }
                Op::Store(Store { dst, src }) => {
                    self.in_scope.store(dst, self.anon[src].clone());
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
                    self.in_scope.add_result(self.anon[src].clone());
                }

                // Allocate values
                Op::Alloc(Alloc { dst, type_id }) => {
                    self.anon[dst] = match BuiltinType::try_from(type_id) {
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
                    self.anon[dst] = match (expected_type_id, &self.anon[src]) {
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
                    if !self.anon[src].as_bool() {
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
        target: AnonymousRegister,
        arg_range: Range<usize>,
        extra_args: Vec<Value>,
    ) -> Result<(), OpError> {
        let func = match &self.anon[target] {
            Value::Function(ptr) => ptr.clone(),
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
            subscope.registers[target_idx].replace(self.anon[src_idx.into()].clone());
        }

        if arg_range.len() < desired_input_args {
            for target_idx in arg_range.len()..desired_input_args {
                subscope.registers[target_idx].replace(other_results.next().unwrap_or_default());
            }
        } else {
            for src_idx in (arg_range.start + desired_input_args)..arg_range.end {
                va_args.push(self.anon[src_idx.into()].clone());
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
                for dst in self.anon.iter_mut().skip(dst_start).take(count) {
                    *dst = results.next().unwrap_or_default();
                }
            }

            Op::SetAllPropertiesFromRet(SetAllPropertiesFromRet { dst, start_idx }) => {
                match &self.anon[dst] {
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
                    _ => todo!("metatables are unsupported"),
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
