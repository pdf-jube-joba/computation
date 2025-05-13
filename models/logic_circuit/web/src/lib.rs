use logic_circuit_core::machine::{Graph, InPin, LogicCircuit, OtPin};
use std::{
    collections::HashMap,
    sync::{LazyLock, Mutex},
};
use wasm_bindgen::prelude::*;

// many global mutable turing machines
static MACHINES: LazyLock<Mutex<Vec<LogicCircuit>>> = LazyLock::new(|| Mutex::new(vec![]));

#[wasm_bindgen]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PinWeb {
    #[wasm_bindgen(getter_with_clone)]
    pub name: String,
    pub state: bool,
}

#[wasm_bindgen]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BoxWeb {
    #[wasm_bindgen(getter_with_clone)]
    pub kind: String,
    #[wasm_bindgen(getter_with_clone)]
    pub inpins: Vec<String>,
    #[wasm_bindgen(getter_with_clone)]
    pub otpins: Vec<PinWeb>,
    // kind が Gate のときのみ Some
    pub state: Option<bool>,
}

#[wasm_bindgen]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EdgeWeb {
    #[wasm_bindgen(getter_with_clone)]
    pub from: String,
    #[wasm_bindgen(getter_with_clone)]
    pub to: String,
}

#[wasm_bindgen]
pub struct GraphWeb {
    #[wasm_bindgen(getter_with_clone)]
    pub boxes: Vec<BoxWeb>,
    #[wasm_bindgen(getter_with_clone)]
    pub inpins: Vec<PinWeb>,
    #[wasm_bindgen(getter_with_clone)]
    pub otpins: Vec<PinWeb>,
    #[wasm_bindgen(getter_with_clone)]
    pub edges: Vec<EdgeWeb>,
}

#[wasm_bindgen]
pub fn new_logic_circuit(code: String) -> usize {
    let mut machines = MACHINES.lock().unwrap();
    machines.len() - 1
}
