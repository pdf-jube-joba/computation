use serde::{Serialize, Serializer};
use turing_machine_core::{
    machine::{CodeEntry as CoreCodeEntry, Sign, Tape, TuringMachineDefinition, TuringMachineSet},
    manipulation,
};
use utils::MealyMachine;

const TAPE_WINDOW: usize = 7;

#[derive(Debug, Clone, Serialize)]
pub struct CodeEntry {
    key_sign: String,
    key_state: String,
    next_sign: String,
    next_state: String,
    direction: String,
}

impl From<CoreCodeEntry> for CodeEntry {
    fn from(entry: CoreCodeEntry) -> Self {
        CodeEntry {
            key_sign: entry.0 .0.to_string(),
            key_state: entry.0 .1.to_string(),
            next_sign: entry.1 .0.to_string(),
            next_state: entry.1 .1.to_string(),
            direction: entry.1 .2.to_string(),
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct TapeView {
    left: [String; TAPE_WINDOW],
    head: String,
    right: [String; TAPE_WINDOW],
}

impl Default for TapeView {
    fn default() -> Self {
        TapeView {
            left: std::array::from_fn(|_| String::new()),
            head: String::new(),
            right: std::array::from_fn(|_| String::new()),
        }
    }
}

impl TapeView {
    fn from_tape(tape: &Tape) -> Self {
        let mut left = std::array::from_fn(|_| String::new());
        let mut right = std::array::from_fn(|_| String::new());
        for (idx, sign) in tape.left.iter().rev().take(TAPE_WINDOW).enumerate() {
            left[idx] = normalize_sign(sign);
        }
        for (idx, sign) in tape.right.iter().rev().take(TAPE_WINDOW).enumerate() {
            right[idx] = normalize_sign(sign);
        }
        TapeView {
            left,
            head: normalize_sign(&tape.head),
            right,
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct Current {
    code: Vec<CodeEntry>,
    now: usize,
    state: String,
    tape: TapeView,
}

#[derive(Debug, Clone, Serialize)]
pub struct Output {
    terminate: bool,
}

#[derive(Debug, Clone)]
pub enum Input {
    Start(Tape),
    Otherwise,
}

impl Serialize for Input {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match self {
            Input::Start(_) => serializer.serialize_str("start"),
            Input::Otherwise => serializer.serialize_str("otherwise"),
        }
    }
}

pub struct TuringMachineWeb {
    definition: TuringMachineDefinition,
    code: Vec<CodeEntry>,
    machine: Option<TuringMachineSet>,
}

impl MealyMachine for TuringMachineWeb {
    type Input = Input;
    type Output = Output;
    type This = Current;

    fn parse_self(input: &str) -> Result<Self, String> {
        let definition =
            manipulation::code::parse_definition(input).map_err(|e| format!("{e:?}"))?;
        let code = definition.code().clone().into_iter().map(CodeEntry::from).collect();
        Ok(TuringMachineWeb {
            definition,
            code,
            machine: None,
        })
    }

    fn parse_input(input: &str) -> Result<Self::Input, String> {
        if input.trim().is_empty() {
            Ok(Input::Otherwise)
        } else {
            parse_tape(input).map(Input::Start)
        }
    }

    fn step(&mut self, input: Self::Input) -> Result<Option<Self::Output>, String> {
        match input {
            Input::Start(tape) => {
                if self.machine.is_some() {
                    return Err("Machine already started".to_string());
                }
                self.machine = Some(TuringMachineSet::new(self.definition.clone(), tape));
            }
            Input::Otherwise => {
                let machine = self
                    .machine
                    .as_mut()
                    .ok_or_else(|| "Machine not started yet".to_string())?;
                if !machine.is_terminate() {
                    machine.step(1).map_err(|_| "Step failed".to_string())?;
                }
            }
        }
        let terminate = self
            .machine
            .as_ref()
            .map(|machine| machine.is_terminate())
            .unwrap_or(false);
        Ok(self.machine.as_ref().map(|_| Output { terminate }))
    }

    fn current(&self) -> Self::This {
        let (now, tape, state) = if let Some(machine) = &self.machine {
            let now = machine
                .next_code()
                .map(|(idx, _)| idx)
                .unwrap_or(self.code.len());
            (
                now,
                TapeView::from_tape(machine.now_tape()),
                machine.now_state().to_string(),
            )
        } else {
            (
                0,
                TapeView::default(),
                self.definition.init_state().to_string(),
            )
        };
        Current {
            code: self.code.clone(),
            now,
            state,
            tape,
        }
    }
}

fn parse_tape(tape: &str) -> Result<Tape, String> {
    let parts: Vec<&str> = tape.split('|').collect();
    if parts.len() != 3 {
        return Err("Invalid tape format | format ... 0,1,2|3|4,5,6".to_string());
    }
    let left: Vec<Sign> = parts
        .first()
        .ok_or_else(|| "Missing left part".to_string())?
        .split(',')
        .map(|s| s.trim().parse().map_err(|e| format!("{e}")))
        .collect::<Result<_, _>>()?;
    let head: Sign = parts[1].trim().parse().map_err(|e| format!("{e}"))?;
    let mut right: Vec<Sign> = parts
        .get(2)
        .ok_or_else(|| "Missing right part".to_string())?
        .split(',')
        .map(|s| s.trim().parse().map_err(|e| format!("{e}")))
        .collect::<Result<_, _>>()?;
    right.reverse();
    Ok(Tape::new(left, head, right))
}

fn normalize_sign(sign: &Sign) -> String {
    let s = sign.to_string();
    if s.trim().is_empty() {
        String::new()
    } else {
        s
    }
}
