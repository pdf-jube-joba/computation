mod machine;
mod parser;
mod validation;

pub use machine::{
    Block, Condition, Environment, Function, LValue, Program, RValue, RecTmIrMachine, Snapshot,
    Stmt,
};
pub use turing_machine::machine::Tape;
pub use validation::validate_no_recursion;

use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

struct Registry {
    functions: HashMap<String, Rc<Function>>,
}

impl Registry {
    fn new() -> Self {
        Self {
            functions: HashMap::new(),
        }
    }
}

thread_local! {
    static REGISTRY: RefCell<Registry> = RefCell::new(Registry::new());
}

pub fn reset_registry() {
    REGISTRY.with(|registry| {
        registry.borrow_mut().functions.clear();
    });
}

pub fn register_function(func: Function) -> Result<Rc<Function>, String> {
    REGISTRY.with(|registry| {
        let mut registry = registry.borrow_mut();
        if let Some(existing) = registry.functions.get(&func.name) {
            if existing.as_ref() == &func {
                return Ok(existing.clone());
            }
            return Err(format!("Function '{}' is defined twice", func.name));
        }
        let func = Rc::new(func);
        registry
            .functions
            .insert(func.name.clone(), func.clone());
        Ok(func)
    })
}

pub fn get_function(name: &str) -> Result<Rc<Function>, String> {
    REGISTRY.with(|registry| {
        let registry = registry.borrow();
        registry
            .functions
            .get(name)
            .cloned()
            .ok_or_else(|| format!("Undefined function '{}'", name))
    })
}

#[cfg(test)]
mod test;
