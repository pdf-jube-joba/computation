use std::sync::{LazyLock, Mutex};
use wasm_bindgen::prelude::*;
use while_minus_lang_core::machine::{
    self, ControlCommand, Environment, ProgramProcess, WhileLanguage, WhileStatement,
};

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

#[wasm_bindgen]
pub fn parse_envs(code: &str) -> Result<Vec<EnvSet>, String> {
    todo!()
}

fn get_machine_by_id(
    id: usize,
) -> Result<std::sync::MutexGuard<'static, Vec<ProgramProcess>>, String> {
    let machines = MACHINES
        .lock()
        .map_err(|_| "Failed to lock machines".to_string())?;
    if id >= machines.len() {
        return Err(format!("No Turing machine found with ID {}", id));
    }
    Ok(machines)
}

#[wasm_bindgen]
pub fn new_while_machine(code: &str, envs: &str) -> Result<usize, String> {
    let mut machines = MACHINES.lock().unwrap();
    let code: WhileLanguage =
        while_minus_lang_core::manipulation::program(code).map_err(|e| format!("{:?}", e))?;
    let env = parse_envs(envs)?
        .into_iter()
        .map(|env| {
            let name: utils::variable::Var = env.name.into();
            let value: utils::number::Number = env.value.into();
            (name, value)
        })
        .collect();
    let envs = Environment { env };
    let machine = ProgramProcess::new(code, envs);
    machines.push(machine);
    Ok(machines.len() - 1)
}

#[wasm_bindgen]
pub fn set_while_machine(id: usize, code: &str, envs: &str) -> Result<(), String> {
    let mut machines = get_machine_by_id(id)?;
    let code: WhileLanguage =
        while_minus_lang_core::manipulation::program(code).map_err(|e| format!("{:?}", e))?;
    let env = parse_envs(envs)?
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
    let program = machines[id].program().statements;
    let mut code = vec![];
    for line in program.iter() {
        match line {
            WhileStatement::Inst(inst) => {
                todo!()
            }
            WhileStatement::Cont(ControlCommand::WhileNotZero(var, statements)) => {
                code.push(format!("while {} {{", var));
                for statement in statements.iter() {
                    code.push(statement.to_string());
                }
                code.push("}".to_string());
            }
        }
    }
    todo!()
}
