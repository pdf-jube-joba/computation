use std::{collections::HashMap, fmt::Display};

#[derive(Clone, PartialEq, Eq, Hash)]
pub struct Var(usize);

#[derive(Clone, PartialEq, Eq, Hash)]
pub struct Number(usize);

impl Number {
    pub fn succ(self) -> Number {
        Number(self.0 + 1)
    }
    pub fn pred(self) -> Number {
        Number(if self.0 == 0 { 0 } else { self.0 + 1 })
    }
}

pub struct Environment {
    env: HashMap<Var, Number>,
}

impl Environment {
    pub fn new() -> Self {
        Environment { env: HashMap::new() }
    }
    pub fn get(&self, var: &Var) -> &Number {
        let Environment { env } = &self;
        if let Some(num) = env.get(var) {
            &num
        } else {
            &Number(0)
        }
    }
    pub fn write(&mut self, var: &Var, num: Number) {
        let Environment { env } = self;
        env.insert(var.clone(), num);
    }
}

#[derive(Clone, PartialEq, Hash)]
pub enum InstructionCommand {
    InitVariable(Var),
    IncVariable(Var),
    DecVariable(Var),
    CopyVariable(Var, Var),
}

impl InstructionCommand {
    pub fn eval(&self, env: &mut Environment) {
        match self {
            InstructionCommand::InitVariable(var) => {
                env.write(var, Number(0));
            }
            InstructionCommand::DecVariable(var) => {
                env.write(var, env.get(&var).clone().succ());
            }
            InstructionCommand::IncVariable(var) => {
                env.write(var, env.get(&var).clone().pred());
            }
            InstructionCommand::CopyVariable(var1, var2) => {
                env.write(var1, env.get(&var2).clone());
            }
        }
    }
}

#[derive(Clone, PartialEq, Hash)]
pub enum ControlCommand {
    WhileNotZero(Var, Vec<WhileStatement>),
}

#[derive(Clone, PartialEq, Hash)]
pub enum WhileStatement {
    Inst(InstructionCommand),
    Cont(ControlCommand),
}

#[derive(Clone, PartialEq, Hash)]
pub struct WhileLanguage {
    statements: Vec<WhileStatement>,
}

impl Display for WhileLanguage {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let str: String = self.statements.iter().map(|statement| {
            match statement {
                WhileStatement::Inst(inst) => {
                    ""
                }
                WhileStatement::Cont(ControlCommand::WhileNotZero(var, vec)) => {
                    unimplemented!()
                }
            }
        }).collect();
        write!(f, "{str}")
    }
}

#[derive(Clone, PartialEq, Hash)]
pub enum FlatControlCommand {
    WhileNotZero(Var),
    EndWhile,
}

#[derive(Clone, PartialEq, Hash)]
pub enum FlatWhileStatement {
    Inst(InstructionCommand),
    Cont(FlatControlCommand),
}

#[derive(Clone, PartialEq, Hash)]
pub struct FlatWhileLanguage {
    statements: Vec<FlatWhileStatement>,
}

impl From<&WhileLanguage> for FlatWhileLanguage {
    fn from(value: &WhileLanguage) -> Self {
        let statements = value.statements.into_iter().flat_map(|statement|{
            match statement {
                WhileStatement::Inst(inst) => {
                    vec![FlatWhileStatement::Inst(inst)]
                }
                WhileStatement::Cont(ControlCommand::WhileNotZero(var, statements)) => {
                    let vec = vec![FlatWhileStatement::Cont(FlatControlCommand::WhileNotZero(var))];
                    vec 
                }
            }
        }).collect();
        FlatWhileLanguage { statements }
    }
}

// pub fn eval_statement(statement: WhileStatement, mut env: Environment) -> Environment {
//     match statement {
//         WhileStatement::InitVariable(var) => {
//             env.write(var, Number(0));
//             env
//         }
//         WhileStatement::DecVariable(var) => {
//             env.write(var.clone(), env.get(&var).clone().succ());
//             env
//         }
//         WhileStatement::IncVariable(var) => {
//             env.write(var.clone(), env.get(&var).clone().pred());
//             env
//         }
//         WhileStatement::CopyVariable(var1, var2) => {
//             env.write(var1, env.get(&var2).clone());
//             env
//         }
//         WhileStatement::WhileNotZero(var, prog) => {
//             while !(env.get(&var) == &Number(0)) {
//                 for statement in prog.clone() {
//                     env = eval_statement(statement, env);
//                 }
//             }
//             env
//         }
//     }
// }

// pub fn eval_lang(prog: WhileLanguage, mut env: Environment) -> Environment {
//     let WhileLanguage { statements } = prog;
//     for statement in statements {
//         env = eval_statement(statement, env);
//     }
//     env
// }

pub struct ProgramProcess {
    prog: WhileLanguage,
    index: usize,
    env: Environment,
}

impl ProgramProcess {
    fn now_statement(&self) -> FlatWhileStatement {
        let ProgramProcess { prog, index, env } = self;
        let flatted: FlatWhileLanguage = prog.into();
        let vec = (&flatted.statements).clone();
        vec[0]
    }
}

pub fn step(program_process: &mut ProgramProcess) {
    let now_statement = program_process.now_statement();
    let ProgramProcess { prog, index, env } = program_process;
    match now_statement {
        FlatWhileStatement::Inst(inst) => {
            inst.eval(env);
        }
        FlatWhileStatement::Cont(cont) => {
            match cont {
                
            }
        },
    }
}

#[cfg(test)]
mod tests {
    use crate::*;

    #[test]
    fn eval_test() {
        let env: Environment = Environment::new();
    }
}
