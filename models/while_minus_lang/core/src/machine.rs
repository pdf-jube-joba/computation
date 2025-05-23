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
    WhileNotZero(Var),
    WhileEnd,
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
    pub fn while_not_zero(var: Var) -> WhileStatement {
        WhileStatement::Cont(ControlCommand::WhileNotZero(var))
    }
    pub fn while_end() -> WhileStatement {
        WhileStatement::Cont(ControlCommand::WhileEnd)
    }
}

impl Display for WhileStatement {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let str: String = match self {
            WhileStatement::Inst(inst) => match inst {
                InstructionCommand::ClearVariable(var) => {
                    format!("clr {var}")
                }
                InstructionCommand::IncVariable(var) => {
                    format!("inc {var}")
                }
                InstructionCommand::DecVariable(var) => {
                    format!("dec {var}")
                }
                InstructionCommand::CopyVariable(var1, var2) => {
                    format!("cpy {var1} <- {var2}")
                }
            },
            WhileStatement::Cont(cont) => match cont {
                ControlCommand::WhileNotZero(var) => {
                    format!("while_nz {var} {{")
                }
                ControlCommand::WhileEnd => "}".to_string(),
            },
        };
        write!(f, "{str}")
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ProgramProcess {
    prog: Vec<WhileStatement>,
    // assert Some(v) => v < prog.statements.len()
    // none means it is terminated
    index: Option<usize>,
    env: Environment,
}

impl ProgramProcess {
    pub fn new(prog: Vec<WhileStatement>, env: Environment) -> Self {
        ProgramProcess {
            prog,
            index: Some(0),
            env,
        }
    }
    pub fn program(&self) -> Vec<WhileStatement> {
        self.prog.clone()
    }
    pub fn current_line(&self) -> Option<usize> {
        self.index
    }
    pub fn env(&self) -> Environment {
        self.env.clone()
    }
    pub fn code(&self) -> Vec<WhileStatement> {
        self.prog.clone()
    }
    pub fn is_terminate(&self) -> bool {
        self.index.is_none()
    }
    pub fn step(&mut self) -> bool {
        let ProgramProcess {
            prog,
            index: index_opt,
            env,
        } = self;
        let Some(index) = index_opt else {
            return false;
        };
        debug_assert!(*index < prog.len());

        let statement = &prog[*index];
        match statement {
            WhileStatement::Inst(inst) => {
                inst.eval(env);
                *index += 1;
                if *index >= prog.len() {
                    *index_opt = None;
                }
            }
            WhileStatement::Cont(cont) => match cont {
                ControlCommand::WhileNotZero(var) => {
                    if env.get(var).is_zero() {
                        // Skip the while loop and move to the matching WhileEnd
                        let mut stack = 1;
                        loop {
                            *index += 1;
                            if *index >= prog.len() {
                                *index_opt = None;
                                return false;
                            }

                            match &prog[*index] {
                                WhileStatement::Cont(ControlCommand::WhileNotZero(_)) => {
                                    stack += 1;
                                }
                                WhileStatement::Cont(ControlCommand::WhileEnd) => {
                                    stack -= 1;
                                    if stack == 0 {
                                        break;
                                    }
                                }
                                _ => {}
                            }
                        }
                        if stack != 0 {
                            *index_opt = None;
                            return false;
                        }
                    } else {
                        // Enter the while loop
                    }
                    *index += 1; // in any case, this works
                    if *index >= prog.len() {
                        *index_opt = None;
                    }
                }
                ControlCommand::WhileEnd => {
                    // Move back to the matching WhileNotZero
                    let mut stack = 1;
                    loop {
                        if *index == 0 {
                            *index_opt = None;
                            return false;
                        }

                        *index -= 1;

                        match &prog[*index] {
                            WhileStatement::Cont(ControlCommand::WhileEnd) => {
                                stack += 1;
                            }
                            WhileStatement::Cont(ControlCommand::WhileNotZero(_)) => {
                                stack -= 1;
                                if stack == 0 {
                                    break;
                                }
                            }
                            _ => {}
                        }
                    }
                }
            },
        };
        true
    }
}

pub fn eval(prog: &[WhileStatement], env: Environment) -> Environment {
    // use step function
    let mut process = ProgramProcess::new(prog.to_vec(), env);
    while !process.is_terminate() {
        process.step();
    }
    process.env()
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
        let prog: Vec<WhileStatement> = vec![WhileStatement::inc(Var::U(0))];
        let env_res = eval(&prog, env.clone());
        let env_exp: Environment = vec![(Var::U(0), Number(1))].into();
        assert_eq!(env_exp, env_res);

        let env: Environment = Environment::new();
        let prog: Vec<WhileStatement> = vec![
            WhileStatement::inc(Var::U(0)),
            WhileStatement::inc(Var::U(0)),
            WhileStatement::inc(Var::U(0)),
            WhileStatement::cpy(Var::U(1), Var::U(0)),
            WhileStatement::cpy(Var::U(0), Var::U(2)),
        ];
        let env_res = eval(&prog, env.clone());
        let env_exp: Environment = vec![(Var::U(1), Number(3))].into();
        assert_eq!(env_exp, env_res);
    }
    #[test]

    fn eval_test_while() {
        let env: Environment = Environment::new();
        let prog: Vec<WhileStatement> = vec![
            WhileStatement::inc(Var::U(0)),
            WhileStatement::inc(Var::U(0)),
            WhileStatement::inc(Var::U(0)),
            WhileStatement::inc(Var::U(0)),
            WhileStatement::inc(Var::U(0)),
            WhileStatement::while_not_zero(Var::U(0)),
            WhileStatement::dec(Var::U(0)),
            WhileStatement::inc(Var::U(1)),
            WhileStatement::while_end(),
        ];

        let mut process = ProgramProcess::new(prog.clone(), env);
        let mut count = 0;
        while !process.is_terminate() && count < 10000 {
            process.step();
            println!(
                "step: {:?} env: {:?}",
                process.current_line(),
                process.env()
            );
            count += 1;
        }
    }
}
