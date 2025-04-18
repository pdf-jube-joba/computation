use logic_circuit_core::machine::{InPin, LogicCircuit, OtPin};
use std::sync::{LazyLock, Mutex};
use wasm_bindgen::prelude::*;

// many global mutable turing machines
static MACHINES: LazyLock<Mutex<Vec<LogicCircuit>>> = LazyLock::new(|| Mutex::new(vec![]));

#[wasm_bindgen]
pub struct PinWeb {
    name: String,
    value: bool,
}

#[wasm_bindgen]
impl PinWeb {
    #[wasm_bindgen(getter)]
    pub fn name(&self) -> String {
        self.name.clone()
    }
    #[wasm_bindgen(getter)]
    pub fn value(&self) -> bool {
        self.value
    }
}

#[wasm_bindgen]
pub struct LogicCircuitWeb {
    circuit: LogicCircuit,
}

#[wasm_bindgen]
impl LogicCircuitWeb {
    #[wasm_bindgen(getter)]
    pub fn name(&self) -> String {
        self.circuit.get_name().to_string()
    }
    #[wasm_bindgen(getter)]
    pub fn get_input(&self) -> Vec<PinWeb> {
        self.circuit
            .get_inpins()
            .iter()
            .map(|v| PinWeb {
                name: v.0.to_string(),
                value: v.1.into(),
            })
            .collect()
    }
    #[wasm_bindgen]
    pub fn get_output(&self) -> Vec<PinWeb> {
        self.circuit
            .get_otpins()
            .iter()
            .map(|v| PinWeb {
                name: v.0.to_string(),
                value: v.1.into(),
            })
            .collect()
    }
}

#[wasm_bindgen]
pub fn new_logic_circuit(code: &str) -> Result<usize, String> {
    let loc = logic_circuit_core::manipulation::parse_main(code).map_err(|e| e.to_string())?;
    let mut machines = MACHINES.lock().unwrap();
    machines.push(loc);
    Ok(machines.len() - 1)
}

#[wasm_bindgen]
pub fn set_logic_circuit(index: usize, code: &str) -> Result<(), String> {
    let loc = logic_circuit_core::manipulation::parse_main(code).map_err(|e| e.to_string())?;
    let mut machines = MACHINES.lock().unwrap();
    if index < machines.len() {
        machines[index] = loc;
        Ok(())
    } else {
        Err("Index out of bounds".to_string())
    }
}
