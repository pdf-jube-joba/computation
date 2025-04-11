use std::fmt::Display;
use utils::number::*;
use utils::variable::Var; // Updated to use Var from utils/variable

#[derive(Debug, Default, Clone)]
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

    pub fn all_written_var(&self) -> Vec<Var> {
        let mut vec: Vec<Var> = self.env.iter().map(|(v, _)| v.clone()).collect();
        vec.sort();
        vec
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
        all_vars.sort();
        all_vars.dedup();

        all_vars.iter().all(|var| self.get(var) == other.get(var))
    }
}

impl From<Vec<(Var, Number)>> for Environment {
    fn from(value: Vec<(Var, Number)>) -> Self {
        Environment { env: value }
    }
}

#[derive(Debug, Clone, PartialEq, Hash)]
pub enum InstructionCommand {
    ClearVariable(Var),
    IncVariable(Var),
    DecVariable(Var),
    CopyVariable(Var, Var),
}

impl InstructionCommand {
    pub fn eval(&self, env: &mut Environment) {
        match self {
            InstructionCommand::ClearVariable(var) => {
                env.write(var, Number(0));
            }
            InstructionCommand::IncVariable(var) => {
                env.write(var, env.get(var).clone().succ());
            }
            InstructionCommand::DecVariable(var) => {
                env.write(var, env.get(var).clone().pred());
            }
            InstructionCommand::CopyVariable(var1, var2) => {
                env.write(var1, env.get(var2).clone());
            }
        }
    }
    pub fn changed_var(&self) -> Var {
        match self {
            InstructionCommand::ClearVariable(var) => var.clone(),
            InstructionCommand::IncVariable(var) => var.clone(),
            InstructionCommand::DecVariable(var) => var.clone(),
            InstructionCommand::CopyVariable(var1, _var2) => var1.clone(),
        }
    }
    pub fn used_var(&self) -> Vec<Var> {
        match self {
            InstructionCommand::ClearVariable(var) => vec![var.clone()],
            InstructionCommand::IncVariable(var) => vec![var.clone()],
            InstructionCommand::DecVariable(var) => vec![var.clone()],
            InstructionCommand::CopyVariable(var1, var2) => vec![var1.clone(), var2.clone()],
        }
    }
}

#[derive(Debug, Clone, PartialEq, Hash)]
pub enum ControlCommand {
    WhileNotZero(Var, Vec<WhileStatement>),
}

#[derive(Debug, Clone, PartialEq, Hash)]
pub enum WhileStatement {
    Inst(InstructionCommand),
    Cont(ControlCommand),
}

impl WhileStatement {
    pub fn clr(var: Var) -> WhileStatement {
        WhileStatement::Inst(InstructionCommand::ClearVariable(var))
    }
    pub fn inc(var: Var) -> WhileStatement {
        WhileStatement::Inst(InstructionCommand::IncVariable(var))
    }
    pub fn dec(var: Var) -> WhileStatement {
        WhileStatement::Inst(InstructionCommand::DecVariable(var))
    }
    pub fn cpy(var1: Var, var2: Var) -> WhileStatement {
        WhileStatement::Inst(InstructionCommand::CopyVariable(var1, var2))
    }
    pub fn while_not_zero(var: Var, vec: Vec<WhileStatement>) -> WhileStatement {
        WhileStatement::Cont(ControlCommand::WhileNotZero(var, vec))
    }
    pub fn used_var(&self) -> Vec<Var> {
        match self {
            WhileStatement::Inst(inst) => inst.used_var(),
            WhileStatement::Cont(ControlCommand::WhileNotZero(_var, statements)) => statements
                .iter()
                .flat_map(|statement| statement.used_var())
                .collect(),
        }
    }
    pub fn changed_var(&self) -> Vec<Var> {
        match self {
            WhileStatement::Inst(inst) => vec![inst.changed_var()],
            WhileStatement::Cont(ControlCommand::WhileNotZero(_var, statements)) => statements
                .iter()
                .flat_map(|statement| statement.changed_var())
                .collect(),
        }
    }
}

impl Display for WhileStatement {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let str: String = match self {
            WhileStatement::Inst(inst) => match inst {
                InstructionCommand::ClearVariable(var) => {
                    format!("clr {var:?} \n")
                }
                InstructionCommand::IncVariable(var) => {
                    format!("inc {var:?} \n")
                }
                InstructionCommand::DecVariable(var) => {
                    format!("dec {var:?} \n")
                }
                InstructionCommand::CopyVariable(var1, var2) => {
                    format!("cpy {var1:?} {var2:?} \n")
                }
            },
            WhileStatement::Cont(ControlCommand::WhileNotZero(var, vec)) => {
                format!(
                    "while_nz {var:?} {{ \n{} }}",
                    vec.iter().map(|stm| format!("{stm}")).collect::<String>()
                )
            }
        };
        write!(f, "{str}")
    }
}

pub fn eval_statement(statement: &WhileStatement, mut env: Environment) -> Environment {
    match statement {
        WhileStatement::Inst(inst) => {
            inst.eval(&mut env);
            env
        }
        WhileStatement::Cont(cont) => match cont {
            ControlCommand::WhileNotZero(var, statements) => {
                while *env.get(var) != Number(0) {
                    for statement in statements {
                        env = eval_statement(statement, env);
                    }
                }
                env
            }
        },
    }
}

#[derive(Debug, Clone, PartialEq, Hash)]
pub struct WhileLanguage {
    pub statements: Vec<WhileStatement>,
}

impl WhileLanguage {
    pub fn new(v: Vec<WhileStatement>) -> Self {
        WhileLanguage { statements: v }
    }
    pub fn changed_var(&self) -> Vec<Var> {
        (self.statements)
            .iter()
            .flat_map(|statement| statement.changed_var())
            .collect()
    }
}

impl From<Vec<WhileStatement>> for WhileLanguage {
    fn from(value: Vec<WhileStatement>) -> Self {
        WhileLanguage { statements: value }
    }
}

impl From<WhileLanguage> for Vec<WhileStatement> {
    fn from(value: WhileLanguage) -> Self {
        value.statements
    }
}

pub fn eval(prog: &WhileLanguage, mut env: Environment) -> Environment {
    let WhileLanguage { statements } = prog;
    for statement in statements {
        env = eval_statement(statement, env);
    }
    env
}

#[derive(Debug, Clone, PartialEq)]
pub struct ProgramProcess {
    prog: WhileLanguage,
    index: usize,
    env: Environment,
}

impl ProgramProcess {
    pub fn new(prog: WhileLanguage, env: Environment) -> Self {
        ProgramProcess {
            prog,
            index: 0,
            env,
        }
    }
    pub fn program(&self) -> WhileLanguage {
        self.prog.clone()
    }
    pub fn current_line(&self) -> usize {
        self.index
    }
    pub fn env(&self) -> Environment {
        self.env.clone()
    }
    pub fn code(&self) -> WhileLanguage {
        self.prog.clone()
    }
    pub fn is_terminate(&self) -> bool {
        self.index == self.prog.statements.len()
    }
    pub fn now_statement(&self) -> WhileStatement {
        let ProgramProcess {
            prog,
            index,
            env: _,
        } = self;
        let mut vec = (prog.statements).clone();
        vec.remove(*index)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn env_eq_tes() {
        let env1: Environment = vec![].into();
        let env2: Environment = vec![].into();
        assert_eq!(env1, env2);

        let env1: Environment = vec![(Var::U(0), Number(1))].into();
        let env2: Environment = vec![(Var::U(0), Number(1))].into();
        assert_eq!(env1, env2);

        let env1: Environment = vec![(Var::U(0), Number(1)), (Var::U(0), Number(1))].into();
        let env2: Environment = vec![(Var::U(0), Number(1))].into();
        assert_eq!(env1, env2);

        let env1: Environment = vec![(Var::U(0), Number(1)), (Var::U(1), Number(2))].into();
        let env2: Environment = vec![(Var::U(1), Number(2)), (Var::U(0), Number(1))].into();
        assert_eq!(env1, env2);

        let env1: Environment = vec![(Var::U(0), Number(0))].into();
        let env2: Environment = vec![].into();
        assert_eq!(env1, env2);
    }

    #[test]
    fn eval_test() {
        let env: Environment = Environment::new();
        let prog: WhileLanguage = vec![WhileStatement::inc(Var::U(0))].into();
        let env_res = eval(&prog, env.clone());
        let env_exp: Environment = vec![(Var::U(0), Number(1))].into();
        assert_eq!(env_exp, env_res);

        let env: Environment = Environment::new();
        let prog: WhileLanguage = vec![
            WhileStatement::inc(Var::U(0)),
            WhileStatement::inc(Var::U(0)),
            WhileStatement::inc(Var::U(0)),
            WhileStatement::cpy(Var::U(1), Var::U(0)),
            WhileStatement::cpy(Var::U(0), Var::U(2)),
        ]
        .into();
        let env_res = eval(&prog, env.clone());
        let env_exp: Environment = vec![(Var::U(1), Number(3))].into();
        assert_eq!(env_exp, env_res);

        let env: Environment = Environment::new();
        let prog: WhileLanguage = vec![
            WhileStatement::inc(Var::U(0)),
            WhileStatement::inc(Var::U(0)),
            WhileStatement::inc(Var::U(0)),
            WhileStatement::inc(Var::U(0)),
            WhileStatement::inc(Var::U(0)),
            WhileStatement::while_not_zero(
                Var::U(0),
                vec![
                    WhileStatement::dec(Var::U(0)),
                    WhileStatement::inc(Var::U(1)),
                ],
            ),
        ]
        .into();
        let env_res = eval(&prog, env.clone());
        let env_exp: Environment = vec![(Var::U(0), Number(0)), (Var::U(1), Number(5))].into();
        assert_eq!(env_exp, env_res);
    }
}
