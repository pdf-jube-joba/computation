use std::sync::{LazyLock, Mutex};
use wasm_bindgen::prelude::*;
use while_minus_lang_core::machine::ProgramProcess;

// many global mutable while language programs
static MACHINES: LazyLock<Mutex<Vec<while_minus_lang_core::machine::ProgramProcess>>> =
    LazyLock::new(|| Mutex::new(vec![]));

#[wasm_bindgen]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProgramState {
    program: Vec<String>,
    current_line: usize,
    env: Vec<(String, usize)>,
}

impl From<ProgramProcess> for ProgramState {
    fn from(process: ProgramProcess) -> Self {
        let program = process
            .program()
            .statements
            .iter()
            .map(|line| line.to_string())
            .collect();
        let current_line = process.current_line();
        let env = process
            .env()
            .env
            .iter()
            .map(|(name, value)| {
                let s: String = name.to_string();
                let v: usize = value.clone().into();
                (s, v)
            })
            .collect();
        Self {
            program,
            current_line,
            env,
        }
    }
}

#[wasm_bindgen]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EnvSet {
    name: String,
    value: usize,
}

#[wasm_bindgen]
impl ProgramState {
    #[wasm_bindgen(getter)]
    pub fn program(&self) -> Vec<String> {
        self.program.clone()
    }

    #[wasm_bindgen(getter)]
    pub fn current_line(&self) -> usize {
        self.current_line
    }

    #[wasm_bindgen(getter)]
    pub fn env(&self) -> Vec<EnvSet> {
        self.env
            .iter()
            .map(|(name, value)| EnvSet {
                name: name.clone(),
                value: *value,
            })
            .collect()
    }
}
