//! Lexical environment, symbol tables, and global/persistent scope management.

use rustlab_math::Value;
use std::cell::RefCell;
use std::collections::{HashMap, HashSet};
use std::rc::Rc;

/// Stores shared global and persistent variable tables across function invocations.
#[derive(Debug, Clone, Default)]
pub struct ScopeContext {
    pub globals: Rc<RefCell<HashMap<String, Value>>>,
    pub persistents: Rc<RefCell<HashMap<String, HashMap<String, Value>>>>,
}

/// Lexical scope mapping variable identifiers to MATLAB values.
#[derive(Debug, Clone)]
pub struct Environment {
    locals: HashMap<String, Value>,
    global_declared: HashSet<String>,
    persistent_declared: HashSet<String>,
    current_function: Option<String>,
    context: ScopeContext,
}

impl Environment {
    /// Creates a new root environment.
    pub fn new() -> Self {
        Self {
            locals: HashMap::new(),
            global_declared: HashSet::new(),
            persistent_declared: HashSet::new(),
            current_function: None,
            context: ScopeContext::default(),
        }
    }

    /// Creates an isolated child environment for a function invocation.
    pub fn child_for_function(&self, function_name: &str) -> Self {
        Self {
            locals: HashMap::new(),
            global_declared: HashSet::new(),
            persistent_declared: HashSet::new(),
            current_function: Some(function_name.to_string()),
            context: self.context.clone(),
        }
    }

    /// Retrieves a variable's value from the local, persistent, or global scope.
    pub fn get(&self, name: &str) -> Option<Value> {
        if self.global_declared.contains(name) {
            return self.context.globals.borrow().get(name).cloned();
        }

        if self.persistent_declared.contains(name) {
            if let Some(ref func) = self.current_function {
                let p = self.context.persistents.borrow();
                if let Some(func_map) = p.get(func) {
                    return func_map.get(name).cloned();
                }
            }
        }

        self.locals.get(name).cloned()
    }

    /// Retrieves a reference to a variable if it is a local variable (not global or persistent).
    pub fn get_local(&self, name: &str) -> Option<&Value> {
        if self.global_declared.contains(name) || self.persistent_declared.contains(name) {
            None
        } else {
            self.locals.get(name)
        }
    }

    /// Sets a variable's value in the appropriate scope.
    pub fn set(&mut self, name: &str, val: Value) {
        if self.global_declared.contains(name) {
            self.context
                .globals
                .borrow_mut()
                .insert(name.to_string(), val);
            return;
        }

        if self.persistent_declared.contains(name) {
            if let Some(ref func) = self.current_function {
                let mut p = self.context.persistents.borrow_mut();
                let func_map = p.entry(func.clone()).or_default();
                func_map.insert(name.to_string(), val);
                return;
            }
        }

        self.locals.insert(name.to_string(), val);
    }

    /// Declares an identifier as global in this scope.
    pub fn declare_global(&mut self, name: &str) {
        self.global_declared.insert(name.to_string());
    }

    /// Declares an identifier as persistent in this function scope.
    pub fn declare_persistent(&mut self, name: &str) {
        self.persistent_declared.insert(name.to_string());
    }

    /// Clears all local variables (equivalent to MATLAB `clear`).
    pub fn clear_locals(&mut self) {
        self.locals.clear();
        self.global_declared.clear();
        self.persistent_declared.clear();
    }

    /// Returns a reference to the local symbol map.
    pub fn locals(&self) -> &HashMap<String, Value> {
        &self.locals
    }
}

impl Default for Environment {
    fn default() -> Self {
        Self::new()
    }
}
