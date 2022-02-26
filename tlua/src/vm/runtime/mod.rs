use std::collections::HashMap;

use tlua_compiler::Chunk;
use tlua_strings::LuaString;
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

pub use tracing_rc::rc::Gc;

pub use self::value::{
    Function,
    Table,
    Value,
};

#[derive(Debug, Default)]
pub struct Runtime {
    globals: HashMap<LuaString, Value>,
}

impl Runtime {
    /// Registers a value associated with a global variable which will be
    /// available to LUA code executed with this runtime.
    pub fn register_global(&mut self, name: &str, value: impl Into<Value>) {
        self.globals.insert(name.into(), value.into());
    }

    /// Reads the value associated with a global variable.
    pub fn load_global(&self, name: &str) -> Option<&Value> {
        self.globals.get(name)
    }

    /// Execute the provided chunk & run it until it completes or returns an
    /// error.
    pub fn execute(&mut self, chunk: &Chunk) -> Result<Vec<Value>, LuaError> {
        let global_scope = Scope::new(chunk.globals_map.len());

        for (ident, value) in self.globals.iter() {
            if let Some(register) = chunk
                .strings
                .lookup_ident(ident)
                .and_then(|ident| chunk.globals_map.get(&ident))
            {
                global_scope.registers[*register].replace(value.clone());
            }
        }

        let current = Scope::new(chunk.main.local_registers);

        let available_scope = vec![global_scope.clone()];
        let execution_context =
            execution_context::Context::new(ScopeSet::new(available_scope, current, vec![]), chunk);

        let result = execution_context.execute()?;

        // TODO(perf): This can consume all the values if we force globals_map to be
        // ordered (e.g. with indexmap).
        let values = global_scope.into_values();
        for (&ident, &idx) in chunk.globals_map.iter() {
            self.globals.insert(
                chunk.strings.get_ident(ident).expect("Valid ident").clone(),
                values[idx].borrow().clone(),
            );
        }

        collect_full();

        Ok(result)
    }
}
