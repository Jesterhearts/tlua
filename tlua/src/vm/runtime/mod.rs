use std::collections::HashMap;

use tlua_compiler::Chunk;
use tlua_parser::ast::identifiers::Ident;
use tracing_rc::rc::collect_full;

use crate::{
    vm::runtime::value::function::{
        Scope,
        ScopeSet,
    },
    LuaError,
};

pub mod execution_context;
pub mod value;

pub use self::value::Value;

#[derive(Debug, Default)]
pub struct Runtime {
    globals: HashMap<Ident, Value>,
}

impl Runtime {
    /// Registers a value associated with a global variable which will be
    /// available to LUA code executed with this runtime.
    pub fn register_global(&mut self, name: &str, value: impl Into<Value>) {
        self.globals.insert(name.into(), value.into());
    }

    /// Execute the provided chunk & run it until it completes or returns an
    /// error.
    pub fn execute(&mut self, chunk: &Chunk) -> Result<Vec<Value>, LuaError> {
        let global_scope = Scope::new(chunk.globals_map.len());

        for (ident, value) in self.globals.iter() {
            if let Some(register) = chunk.globals_map.get(ident) {
                global_scope.registers[*register].replace(value.clone());
            }
        }

        let current = Scope::new(chunk.main.local_registers);
        let anon = vec![Value::Nil; chunk.main.anon_registers];

        let global_scope = vec![global_scope];
        let execution_context = execution_context::Context::new(
            ScopeSet::new(global_scope, current, anon, vec![]),
            chunk,
        );

        let result = execution_context.execute()?;
        collect_full();

        Ok(result)
    }
}
