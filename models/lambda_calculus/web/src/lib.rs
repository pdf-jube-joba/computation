use lambda_calculus_core::machine::{is_normal, left_most_reduction, LambdaTerm};
use std::sync::{LazyLock, Mutex};
use wasm_bindgen::prelude::*;

// global mutable lambda caluclus terms
static MACHINES: LazyLock<Mutex<Vec<LambdaTerm>>> = LazyLock::new(|| Mutex::new(vec![]));

#[wasm_bindgen]
#[derive(Debug, Clone)]
pub struct LambdaTermWasm {
    term: LambdaTerm,
}

#[wasm_bindgen]
impl LambdaTermWasm {
    #[wasm_bindgen(getter)]
    pub fn into_string(&self) -> String {
        self.term.to_string()
    }
}

#[wasm_bindgen]
pub fn parse_lambda(term: &str) -> Result<LambdaTermWasm, String> {
    let code = lambda_calculus_core::manipulation::parse::parse_lambda_read_to_end(term)?;
    Ok(LambdaTermWasm { term: code })
}

fn get_machine() -> Result<std::sync::MutexGuard<'static, Vec<LambdaTerm>>, String> {
    let machines = MACHINES
        .lock()
        .map_err(|_| "Failed to lock machines".to_string())?;
    Ok(machines)
}

#[wasm_bindgen]
pub fn new_lambda_term(term: &LambdaTermWasm) -> Result<usize, String> {
    let mut machines = MACHINES
        .lock()
        .map_err(|_| "Failed to lock machines".to_string())?;
    let term = term.term.clone();
    machines.push(term);
    Ok(machines.len() - 1)
}

#[wasm_bindgen]
pub fn set_lambda_term(id: usize, term: &LambdaTermWasm) -> Result<(), String> {
    let mut machines = get_machine()?;
    machines[id] = term.term.clone();
    Ok(())
}

#[wasm_bindgen]
pub fn get_lambda_term(id: usize) -> Result<LambdaTermWasm, String> {
    let machines = get_machine()?;
    if id >= machines.len() {
        return Err(format!("Machine with id {} not found", id));
    }
    let term = &machines[id];
    Ok(LambdaTermWasm { term: term.clone() })
}

#[wasm_bindgen]
pub fn step_lambda_term(id: usize) -> Result<(), String> {
    let mut machines = get_machine()?;
    if id >= machines.len() {
        return Err(format!("Machine with id {} not found", id));
    }
    let term = &mut machines[id];
    // check if term is normal
    if is_normal(term) {
        return Err("Term is already in normal form".to_string());
    }
    // left most reduction
    let next: LambdaTerm = left_most_reduction(term.clone());
    *term = next;
    Ok(())
}
