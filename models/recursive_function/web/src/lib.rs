use recursive_function_core::machine::Process;
use std::sync::{LazyLock, Mutex};
use wasm_bindgen::prelude::*;

// many global mutable turing machines
static MACHINES: LazyLock<Mutex<Vec<Process>>> = LazyLock::new(|| Mutex::new(vec![]));

#[wasm_bindgen]
pub struct ProcessWeb {
    process: Process,
}

#[wasm_bindgen]
impl ProcessWeb {
    #[wasm_bindgen(getter)]
    pub fn into_string(&self) -> String {
        self.process.to_string()
    }
}

// #[wasm_bindgen]
// pub fn 
