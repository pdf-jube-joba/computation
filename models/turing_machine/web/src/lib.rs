use std::sync::{LazyLock, Mutex};
use turing_machine_core::machine::{
    Direction, Sign, State, Tape, TuringMachineDefinition, TuringMachineSet,
};
use wasm_bindgen::prelude::*;

// many global mutable tapes
static MACHINES: LazyLock<Mutex<Vec<TuringMachineSet>>> = LazyLock::new(|| Mutex::new(vec![]));

#[wasm_bindgen]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TapeForWeb {
    left: Vec<String>,
    head: String,
    right: Vec<String>,
}

#[wasm_bindgen]
impl TapeForWeb {
    #[wasm_bindgen(constructor)]
    pub fn new(left: Vec<String>, head: String, right: Vec<String>) -> Self {
        TapeForWeb { left, head, right }
    }

    #[wasm_bindgen(getter)]
    pub fn left(&self) -> Vec<String> {
        self.left.clone()
    }

    #[wasm_bindgen(getter)]
    pub fn head(&self) -> String {
        self.head.clone()
    }

    #[wasm_bindgen(getter)]
    pub fn right(&self) -> Vec<String> {
        self.right.clone()
    }
}

impl TryFrom<TapeForWeb> for Tape {
    type Error = String;

    fn try_from(tape_for_web: TapeForWeb) -> Result<Self, Self::Error> {
        let left = tape_for_web
            .left
            .into_iter()
            .map(|s| s.as_str().try_into())
            .collect::<Result<Vec<Sign>, String>>()?;
        let head = tape_for_web.head.as_str().try_into()?;
        let right = tape_for_web
            .right
            .into_iter()
            .map(|s| s.as_str().try_into())
            .collect::<Result<Vec<Sign>, String>>()?;
        Ok(Tape::new(left, head, right))
    }
}

impl From<Tape> for TapeForWeb {
    fn from(tape: Tape) -> Self {
        TapeForWeb {
            left: tape.left.into_iter().map(|s| s.to_string()).collect(),
            head: tape.head.to_string(),
            right: tape.right.into_iter().map(|s| s.to_string()).collect(),
        }
    }
}

#[wasm_bindgen]
pub fn parse_tape(tape: &str) -> Result<TapeForWeb, String> {
    let parts: Vec<&str> = tape.split("|").collect();
    if parts.len() != 3 {
        return Err("Invalid tape format | format ... 0,1,2|3|4,5,6".to_string());
    }
    let left = parts[0].split(',').map(|s| s.trim().to_string()).collect();
    let head = parts[1].to_string();
    let right = parts[2].split(',').map(|s| s.trim().to_string()).collect();
    Ok(TapeForWeb::new(left, head, right))
}

#[wasm_bindgen]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CodeEntry {
    key_sign: String,
    key_state: String,
    next_sign: String,
    next_state: String,
    direction: String,
}

#[wasm_bindgen]
impl CodeEntry {
    #[wasm_bindgen(constructor)]
    pub fn new(
        key_sign: String,
        key_state: String,
        next_sign: String,
        next_state: String,
        direction: String,
    ) -> Self {
        CodeEntry {
            key_sign,
            key_state,
            next_sign,
            next_state,
            direction,
        }
    }

    #[wasm_bindgen(getter)]
    pub fn key_sign(&self) -> String {
        self.key_sign.clone()
    }

    #[wasm_bindgen(getter)]
    pub fn key_state(&self) -> String {
        self.key_state.clone()
    }

    #[wasm_bindgen(getter)]
    pub fn next_sign(&self) -> String {
        self.next_sign.clone()
    }

    #[wasm_bindgen(getter)]
    pub fn next_state(&self) -> String {
        self.next_state.clone()
    }

    #[wasm_bindgen(getter)]
    pub fn direction(&self) -> String {
        self.direction.clone()
    }
}

#[wasm_bindgen]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Code {
    init_state: String,
    accepted_state: Vec<String>,
    code: Vec<CodeEntry>,
}

#[wasm_bindgen]
impl Code {
    #[wasm_bindgen(getter)]
    pub fn init_state(&self) -> String {
        self.init_state.clone()
    }

    #[wasm_bindgen(getter)]
    pub fn accepted_state(&self) -> Vec<String> {
        self.accepted_state.clone()
    }

    #[wasm_bindgen(getter)]
    pub fn code(&self) -> Vec<CodeEntry> {
        self.code.clone()
    }
}

#[wasm_bindgen]
pub fn parse_code(str: &str) -> Result<Code, String> {
    // get init state from first line
    let mut lines = str.lines();

    let init_state = lines
        .next()
        .ok_or_else(|| "Missing initial states line: first line should be init_state".to_string())?
        .trim()
        .to_string();

    let accepted_state = lines
        .next()
        .ok_or_else(|| {
            "Missing accepted states line: second line should be accepted states split by ','"
                .to_string()
        })?
        .split(',')
        .map(|s| s.trim().to_string())
        .collect::<Vec<_>>();

    let code = lines
        .map(|line| {
            let parts: Vec<&str> = line.split(",").collect();
            if parts.len() != 5 {
                return Err(format!("Invalid code entry: {}", line));
            }
            Ok(CodeEntry::new(
                parts[0].trim().to_string(),
                parts[1].trim().to_string(),
                parts[2].trim().to_string(),
                parts[3].trim().to_string(),
                parts[4].trim().to_string(),
            ))
        })
        .collect::<Result<Vec<_>, _>>()?;

    let entire_code = Code {
        init_state,
        accepted_state,
        code,
    };

    let _ = construct_turing_machine_definition(entire_code.clone())?;

    Ok(entire_code)
}

/// Helper function to construct a TuringMachineDefinition
fn construct_turing_machine_definition(code: Code) -> Result<TuringMachineDefinition, String> {
    let init_state: State = code.init_state.as_str().try_into()?;
    let accepted_state: Vec<State> = code
        .accepted_state
        .into_iter()
        .map(|s| s.as_str().try_into())
        .collect::<Result<_, String>>()?;
    let code: Vec<(_, _)> = code
        .code
        .into_iter()
        .map(|entry| {
            let s: ((Sign, State), (Sign, State, Direction)) = (
                (
                    entry.key_sign.as_str().try_into()?,
                    entry.key_state.as_str().try_into()?,
                ),
                (
                    entry.next_sign.as_str().try_into()?,
                    entry.next_state.as_str().try_into()?,
                    entry.direction.as_str().try_into()?,
                ),
            );
            Ok(s)
        })
        .collect::<Result<_, String>>()?;
    TuringMachineDefinition::new(init_state, accepted_state, code)
}

/// Helper function to lock `MACHINES` and retrieve the machine by `id`.
fn get_machine_by_id(
    id: usize,
) -> Result<std::sync::MutexGuard<'static, Vec<TuringMachineSet>>, String> {
    let machines = MACHINES
        .lock()
        .map_err(|_| "Failed to lock MACHINES".to_string())?;

    if id >= machines.len() {
        return Err(format!("No Turing machine found with ID {}", id));
    }

    Ok(machines)
}

// make a new Turing machine and add it to the global list
// return the index of the new machine
#[wasm_bindgen]
pub fn new_turing_machine(code: &Code, tape: &TapeForWeb) -> Result<usize, String> {
    let definition = construct_turing_machine_definition(code.clone())?;
    let tmset: TuringMachineSet = TuringMachineSet::new(definition, tape.clone().try_into()?);

    let mut machines = MACHINES
        .lock()
        .map_err(|_| "Failed to lock MACHINES".to_string())?;

    machines.push(tmset);
    Ok(machines.len() - 1)
}

// set a Turing machine by given id
#[wasm_bindgen]
pub fn set_turing_machine(id: usize, code: &Code, tape: &TapeForWeb) -> Result<(), String> {
    let definition = construct_turing_machine_definition(code.clone())?;

    let mut machines = get_machine_by_id(id)?;
    machines[id] = TuringMachineSet::new(definition, tape.clone().try_into()?);

    Ok(())
}

#[wasm_bindgen]
pub fn get_code(id: usize) -> Result<Vec<CodeEntry>, String> {
    let machines = get_machine_by_id(id)?;

    let code = &machines[id].code();
    let code_entries: Vec<CodeEntry> = code
        .iter()
        .map(
            |((key_sign, key_state), (next_sign, next_state, direction))| CodeEntry {
                key_sign: key_sign.to_string(),
                key_state: key_state.to_string(),
                next_sign: next_sign.to_string(),
                next_state: next_state.to_string(),
                direction: direction.to_string(),
            },
        )
        .collect();

    Ok(code_entries)
}

#[wasm_bindgen]
pub fn get_initial_state(id: usize) -> Result<String, String> {
    let machines = get_machine_by_id(id)?;
    let initial_state = machines[id].init_state();
    Ok(initial_state.to_string())
}

#[wasm_bindgen]
pub fn get_accepted_state(id: usize) -> Result<Vec<String>, String> {
    let machines = get_machine_by_id(id)?;
    let accepted_state = machines[id].accepted_state();
    Ok(accepted_state.iter().map(|s| s.to_string()).collect())
}

#[wasm_bindgen]
pub fn get_now_tape(id: usize) -> Result<TapeForWeb, String> {
    let machines = get_machine_by_id(id)?;
    let tape = machines[id].now_tape();
    Ok(tape.clone().into())
}

#[wasm_bindgen]
pub fn get_now_state(id: usize) -> Result<String, String> {
    let machines = get_machine_by_id(id)?;
    let state = machines[id].now_state();
    Ok(state.to_string())
}

#[wasm_bindgen]
pub fn step_machine(id: usize) -> Result<(), String> {
    let mut machines = get_machine_by_id(id)?;
    let machine = &mut machines[id];
    machine.step(1).map_err(|_| "Step failed".to_string())?;
    Ok(())
}

#[wasm_bindgen]
pub fn get_next(id: usize) -> Result<usize, String> {
    let machines = get_machine_by_id(id)?;
    let next = machines[id]
        .next_code()
        .ok_or_else(|| "No next code available".to_string())?;
    Ok(next.0)
}

#[wasm_bindgen]
pub fn next_direction(id: usize) -> Result<String, String> {
    let machines = get_machine_by_id(id)?;
    let next = machines[id]
        .next_code()
        .ok_or_else(|| "No next code available".to_string())?;
    Ok(next.1 .2.to_string())
}
