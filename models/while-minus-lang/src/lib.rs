use std::collections::HashMap;

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

#[derive(Clone, PartialEq, Hash)]
pub enum WhileStatement {
    InitVariable(Var),
    IncVariable(Var),
    DecVariable(Var),
    CopyVariable(Var, Var),
    WhileNotZero(Var, WhileLanguage),
}

#[derive(Clone, PartialEq, Hash)]
pub struct WhileLanguage {
    statements: Vec<WhileStatement>,
}

pub struct Environment {
    env: HashMap<Var, Number>,
}

impl Environment {
    pub fn get(&self, var: &Var) -> &Number {
        let Environment { env } = &self;
        if let Some(num) = env.get(var) {
            &num
        } else {
            &Number(0)
        }
    }
    pub fn write(&mut self, var: Var, num: Number) {
        let Environment { env } = self;
        env.insert(var, num);
    }
}

pub fn eval_statement(statement: WhileStatement, mut env: Environment) -> Environment {
    match statement {
        WhileStatement::InitVariable(var) => {
            env.write(var, Number(0));
            env
        }
        WhileStatement::DecVariable(var) => {
            env.write(var.clone(), env.get(&var).clone().succ());
            env
        }
        WhileStatement::IncVariable(var) => {
            env.write(var.clone(), env.get(&var).clone().pred());
            env
        }
        WhileStatement::CopyVariable(var1, var2) => {
            env.write(var1, env.get(&var2).clone());
            env
        }
        WhileStatement::WhileNotZero(var, prog) => {
            while !(env.get(&var) == &Number(0)) {
                env = eval_lang(prog.clone(), env);
            }
            env
        }
    }
}

pub fn eval_lang(prog: WhileLanguage, mut env: Environment) -> Environment {
    let WhileLanguage { statements } = prog;
    for statement in statements {
        env = eval_statement(statement, env);
    }
    env
}
