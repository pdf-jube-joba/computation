use serde::Serialize;
use utils::Machine;
use utils::number::Number;
use utils::variable::Var;

#[derive(Debug, Default, Clone, Serialize)]
pub struct Environment {
    pub env: Vec<(Var, Number)>,
}

impl Environment {
    pub fn new() -> Self {
        Environment { env: vec![] }
    }

    pub fn get(&self, var: &Var) -> &Number {
        self.env
            .iter()
            .find(|(v, _)| v == var)
            .map(|(_, num)| num)
            .unwrap_or(&Number(0))
    }

    pub fn write(&mut self, var: &Var, num: Number) {
        if let Some((_, existing_num)) = self.env.iter_mut().find(|(v, _)| v == var) {
            *existing_num = num;
        } else {
            self.env.push((var.clone(), num));
        }
    }
}

impl PartialEq for Environment {
    fn eq(&self, other: &Self) -> bool {
        let mut all_vars: Vec<Var> = self
            .env
            .iter()
            .map(|(v, _)| v.clone())
            .chain(other.env.iter().map(|(v, _)| v.clone()))
            .collect();
        all_vars.dedup();

        all_vars.iter().all(|var| self.get(var) == other.get(var))
    }
}

impl From<Vec<(Var, Number)>> for Environment {
    fn from(value: Vec<(Var, Number)>) -> Self {
        Environment { env: value }
    }
}

#[derive(Debug, Clone, Serialize)]
pub enum Command {
    Clr(Var),
    Inc(Var),
    Dec(Var),
    Cpy(Var, Var),
    Ifz(Var, Number),
}

#[derive(Debug, Clone, Serialize)]
pub struct Program {
    pub commands: Vec<Command>,
    pub pc: Number,
    pub env: Environment,
}

impl Machine for Program {
    type Code = Vec<Command>;
    type AInput = Environment;
    type This = Program;
    type RInput = ();

    type Output = Environment;

    fn parse_code(code: &str) -> Result<Self::Code, String> {
        crate::manipulation::program_read_to_end(code).map_err(|e| e.to_string())
    }

    fn parse_ainput(ainput: &str) -> Result<Self::AInput, String> {
        let v = crate::manipulation::env_read_to_end(ainput).map_err(|e| e.to_string())?;
        Ok(Environment::from(v))
    }

    fn parse_rinput(rinput: &str) -> Result<Self::RInput, String> {
        if rinput.trim().is_empty() {
            Ok(())
        } else {
            Err("This machine does not take any runtime input.".to_string())
        }
    }

    fn make(code: Self::Code, ainput: Self::AInput) -> Result<Self, String> {
        Ok(Program {
            commands: code,
            pc: Number(0),
            env: ainput,
        })
    }

    fn step(&mut self, rinput: Self::RInput) -> Result<Option<Self::Output>, String> {
        if (self.pc).0 as usize >= self.commands.len() {
            return Ok(Some(self.env.clone()));
        }

        let command = &self.commands[(self.pc).0 as usize];
        match command {
            Command::Clr(var) => {
                self.env.write(var, Number(0));
                self.pc.0 += 1;
            }
            Command::Inc(var) => {
                let val = self.env.get(var).0 + 1;
                self.env.write(var, Number(val));
                self.pc.0 += 1;
            }
            Command::Dec(var) => {
                let val = self.env.get(var).0.saturating_sub(1);
                self.env.write(var, Number(val));
                self.pc.0 += 1;
            }
            Command::Cpy(src, dest) => {
                let val = self.env.get(src).clone();
                self.env.write(dest, val);
                self.pc.0 += 1;
            }
            Command::Ifz(var, target) => {
                if self.env.get(var).0 == 0 {
                    self.pc = target.clone();
                } else {
                    self.pc.0 += 1;
                }
            }
        }

        Ok(None)
    }

    fn current(&self) -> Self::This {
        self.clone()
    }
}
