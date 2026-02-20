use serde::Serialize;
use utils::identifier::Identifier;
use utils::number::Number;
use utils::{Machine, TextCodec};

#[derive(Debug, Default, Clone, Serialize)]
pub struct Environment {
    pub env: Vec<(Identifier, Number)>,
}

impl Environment {
    pub fn new() -> Self {
        Environment { env: vec![] }
    }

    pub fn get(&self, var: &Identifier) -> Number {
        self.env
            .iter()
            .find_map(|(v, num)| if v == var { Some(num.clone()) } else { None })
            .unwrap_or_default()
    }

    pub fn write(&mut self, var: &Identifier, num: Number) {
        if let Some((_, existing_num)) = self.env.iter_mut().find(|(v, _)| v == var) {
            *existing_num = num;
        } else {
            self.env.push((var.clone(), num));
        }
    }
}

impl TextCodec for Environment {
    fn parse(text: &str) -> Result<Self, String> {
        let env = crate::manipulation::env_read_to_end(text).map_err(|e| e.to_string())?;
        Ok(Environment { env })
    }

    fn write_fmt(&self, f: &mut impl std::fmt::Write) -> std::fmt::Result {
        for (var, num) in &self.env {
            writeln!(f, "{} = {}", var.as_str(), num.to_decimal_string())?;
        }
        Ok(())
    }
}

impl PartialEq for Environment {
    fn eq(&self, other: &Self) -> bool {
        let mut all_vars: Vec<Identifier> = self
            .env
            .iter()
            .map(|(v, _)| v.clone())
            .chain(other.env.iter().map(|(v, _)| v.clone()))
            .collect();
        all_vars.dedup();

        all_vars.iter().all(|var| self.get(var) == other.get(var))
    }
}

impl From<Vec<(Identifier, Number)>> for Environment {
    fn from(value: Vec<(Identifier, Number)>) -> Self {
        Environment { env: value }
    }
}

#[derive(Debug, Clone, Serialize)]
pub enum Command {
    Clr(Identifier),
    Inc(Identifier),
    Dec(Identifier),
    Cpy(Identifier, Identifier),
    Ifnz(Identifier, Number),
}

#[derive(Debug, Clone, Serialize)]
pub struct Code(pub Vec<Command>);

impl TextCodec for Code {
    fn parse(text: &str) -> Result<Self, String> {
        let code = crate::manipulation::program_read_to_end(text).map_err(|e| e.to_string())?;
        Ok(Code(code))
    }

    fn write_fmt(&self, f: &mut impl std::fmt::Write) -> std::fmt::Result {
        for command in &self.0 {
            let line = match command {
                Command::Clr(var) => format!("clr {}\n", var.as_str()),
                Command::Inc(var) => format!("inc {}\n", var.as_str()),
                Command::Dec(var) => format!("dec {}\n", var.as_str()),
                Command::Cpy(dest, src) => format!("cpy {} {}\n", dest.as_str(), src.as_str()),
                Command::Ifnz(var, target) => format!("ifnz {} {}\n", var.as_str(), target.print()),
            };
            f.write_str(&line)?;
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct Program {
    pub commands: Code,
    pub pc: Number,
    pub env: Environment,
}

impl Machine for Program {
    type Code = Code;
    type AInput = Environment;
    type SnapShot = Program;
    type RInput = ();

    type Output = Environment;

    fn make(code: Self::Code, ainput: Self::AInput) -> Result<Self, String> {
        Ok(Program {
            commands: code,
            pc: 0.into(),
            env: ainput,
        })
    }

    fn step(&mut self, _rinput: Self::RInput) -> Result<Option<Self::Output>, String> {
        let pc = self.pc.as_usize()?;
        if pc >= self.commands.0.len() {
            return Ok(Some(self.env.clone()));
        }

        let command = &self.commands.0[pc];
        match command {
            Command::Clr(var) => {
                self.env.write(var, 0.into());
                self.pc += 1;
            }
            Command::Inc(var) => {
                let val = self.env.get(var) + 1;
                self.env.write(var, val);
                self.pc += 1;
            }
            Command::Dec(var) => {
                let val = self.env.get(var) - 1;
                self.env.write(var, val);
                self.pc += 1;
            }
            Command::Cpy(dest, src) => {
                let val = self.env.get(src).clone();
                self.env.write(dest, val);
                self.pc += 1;
            }
            Command::Ifnz(var, target) => {
                if !self.env.get(var).is_zero() {
                    self.pc = target.clone();
                } else {
                    self.pc += 1;
                }
            }
        }

        Ok(None)
    }

    fn current(&self) -> Self::SnapShot {
        self.clone()
    }
}
