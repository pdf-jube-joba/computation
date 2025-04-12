use std::sync::{LazyLock, Mutex};
use wasm_bindgen::prelude::*;
use while_minus_lang_core::machine::{Environment, ProgramProcess, WhileStatement};

// many global mutable while language programs
static MACHINES: LazyLock<Mutex<Vec<while_minus_lang_core::machine::ProgramProcess>>> =
    LazyLock::new(|| Mutex::new(vec![]));

#[wasm_bindgen]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProgramState {
    program: Vec<String>,
    current_line: Option<usize>,
    env: Vec<(String, usize)>,
}

impl From<ProgramProcess> for ProgramState {
    fn from(process: ProgramProcess) -> Self {
        let program = process
            .program()
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
    pub fn current_line(&self) -> Option<usize> {
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

// string line parse "x=1,y=2"
pub fn parse_env(code: &str) -> Result<Vec<EnvSet>, String> {
    let mut envs = vec![];
    for s in code.split(",") {
        let s = s.trim();
        if s.is_empty() {
            continue;
        }
        let mut parts = s.split("=");
        let name: String = parts
            .next()
            .ok_or_else(|| format!("Invalid env format: {}", s))?
            .trim()
            .to_string();
        let value: usize = parts
            .next()
            .ok_or_else(|| format!("Invalid env format: {}", s))?
            .trim()
            .parse()
            .map_err(|value| format!("Invalid env value: {}", value))?;
        if parts.next().is_some() {
            return Err(format!("Invalid env format: {}", s));
        }
        envs.push(EnvSet { name, value });
    }
    Ok(envs)
}

fn get_machine_by_id(
    id: usize,
) -> Result<std::sync::MutexGuard<'static, Vec<ProgramProcess>>, String> {
    let machines = MACHINES
        .lock()
        .map_err(|_| "Failed to lock machines".to_string())?;
    if id >= machines.len() {
        return Err(format!("No machine found with ID {}", id));
    }
    Ok(machines)
}

#[wasm_bindgen]
pub fn new_while_machine(code: &str, envs: &str) -> Result<usize, String> {
    let code = while_minus_lang_core::manipulation::program_read_to_end(code)
        .map_err(|e| format!("{:?}", e))?;
    let env = parse_env(envs)?
        .into_iter()
        .map(|env| {
            let name: utils::variable::Var = env.name.into();
            let value: utils::number::Number = env.value.into();
            (name, value)
        })
        .collect();
    let envs = Environment { env };
    let mut machines = MACHINES.lock().unwrap();
    let machine = ProgramProcess::new(code, envs);
    machines.push(machine);
    Ok(machines.len() - 1)
}

#[wasm_bindgen]
pub fn set_while_machine(id: usize, code: &str, envs: &str) -> Result<(), String> {
    let mut machines = get_machine_by_id(id)?;
    let code: Vec<WhileStatement> =
        while_minus_lang_core::manipulation::program(code).map_err(|e| format!("{:?}", e))?;
    let env = parse_env(envs)?
        .into_iter()
        .map(|env| {
            let name: utils::variable::Var = env.name.into();
            let value: utils::number::Number = env.value.into();
            (name, value)
        })
        .collect();
    let envs = Environment { env };
    machines[id] = ProgramProcess::new(code, envs);
    Ok(())
}

#[wasm_bindgen]
pub fn get_code(id: usize) -> Result<Vec<String>, String> {
    let machines = get_machine_by_id(id)?;
    let program = machines[id].program();
    let mut code: Vec<String> = vec![];
    for line in program.iter() {
        code.push(line.to_string());
    }
    Ok(code)
}

#[wasm_bindgen]
pub fn get_env(id: usize) -> Result<Vec<EnvSet>, String> {
    let machines = get_machine_by_id(id)?;
    let env = machines[id].env();
    let mut envs: Vec<EnvSet> = vec![];
    for (name, value) in env.env.iter() {
        envs.push(EnvSet {
            name: name.to_string(),
            value: value.clone().into(),
        });
    }
    Ok(envs)
}

#[wasm_bindgen]
pub fn get_current_line(id: usize) -> Result<Option<usize>, String> {
    let machines = get_machine_by_id(id)?;
    let current_line = machines[id].current_line();
    Ok(current_line)
}
