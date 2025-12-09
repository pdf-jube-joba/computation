use serde::Serialize;
use utils::number::Number;
use utils::variable::Var;

#[derive(Debug, Default, Clone, Serialize)]
pub struct Environment {
    pub env: Vec<(Var, Number)>, // Changed from HashMap to Vec
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

pub struct Program {
    pub commands: Vec<Command>,
    pub env: Environment,
    pub pc: Number,
}
