use serde::Serialize;
use utils::OneTime;
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

impl OneTime for Program {
    type Code = Vec<Command>;
    type Input = Environment;
    type Env = Program;

    fn parse_code(input: &str) -> Result<Self::Code, String> {
        let code = crate::manipulation::program_read_to_end(input).map_err(|e| e.to_string())?;
        Ok(code)
    }

    fn parse_input(input: &str) -> Result<Self::Input, String> {
        let env = crate::manipulation::env_read_to_end(input).map_err(|e| e.to_string())?;
        Ok(Environment { env })
    }

    fn setup(code: Self::Code, input: Self::Input) -> Result<Self, String> {
        Ok(Program {
            commands: code,
            pc: Number(0),
            env: input,
        })
    }

    fn run_onestep(&mut self) {
        match self.commands.get(self.pc.0) {
            Some(command) => match command {
                Command::Clr(v) => {
                    self.env.write(v, Number(0));
                    self.pc.0 += 1;
                }
                Command::Inc(v) => {
                    let val = self.env.get(v).clone();
                    self.env.write(v, Number(val.0 + 1));
                    self.pc.0 += 1;
                }
                Command::Dec(v) => {
                    let val = self.env.get(v).clone();
                    if val.0 > 0 {
                        self.env.write(v, Number(val.0 - 1));
                    }
                    self.pc.0 += 1;
                }
                Command::Cpy(v1, v2) => {
                    let val = self.env.get(v2).clone();
                    self.env.write(v1, val);
                    self.pc.0 += 1;
                }
                Command::Ifz(v, n) => {
                    let val = self.env.get(v).clone();
                    if val.0 == 0 {
                        self.pc = n.clone();
                    } else {
                        self.pc.0 += 1;
                    }
                }
            },
            None => {
                // Do nothing if pc is out of bounds
            }
        }
    }

    fn is_terminated(&self) -> bool {
        self.pc.0 >= self.commands.len()
    }

    fn current_env(&self) -> Self::Env {
        self.clone()
    }
}
