use std::{collections::HashMap, fmt::Display};
use utils::number::*;

#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Var(usize);

pub fn new_var(vars: Vec<Var>) -> Var {
    vars.into_iter().max().unwrap_or(0.into())
}

impl From<usize> for Var {
    fn from(value: usize) -> Self {
        Var(value)
    }
}

impl TryFrom<&str> for Var {
    type Error = ();
    fn try_from(value: &str) -> Result<Self, Self::Error> {
        Ok(Var(value.parse().map_err(|_| ())?))
    }
}

#[derive(Debug, Clone)]
pub struct Environment {
    env: HashMap<Var, Number>,
}

impl Environment {
    pub fn new() -> Self {
        Environment {
            env: HashMap::new(),
        }
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
    pub fn all_written_var(&self) -> Vec<Var> {
        let mut vec: Vec<Var> = self.env.keys().into_iter().cloned().collect();
        vec.sort();
        vec
    }
}

impl PartialEq for Environment {
    fn eq(&self, other: &Self) -> bool {
        let Environment { env: env1 } = self;
        let Environment { env: env2 } = other;
        let all_written_var: Vec<Var> = {
            let mut vec: Vec<Var> = vec![];
            vec.extend(env1.keys().into_iter().cloned());
            vec.extend(env2.keys().into_iter().cloned());
            vec
        };
        for var in all_written_var {
            if self.get(&var) != other.get(&var) {
                return false;
            }
        }
        true
    }
}

impl From<Vec<(Var, Number)>> for Environment {
    fn from(value: Vec<(Var, Number)>) -> Self {
        Environment {
            env: HashMap::from_iter(value.into_iter()),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Hash)]
pub enum InstructionCommand {
    InitVariable(Var),
    IncVariable(Var),
    DecVariable(Var),
    CopyVariable(Var, Var),
}

fn change_var(var: Var, map: &HashMap<Var, Var>) -> Var {
    if let Some(var_new) = map.get(&var) {
        var_new.clone()
    } else {
        var
    }
}

impl InstructionCommand {
    pub fn eval(&self, env: &mut Environment) {
        match self {
            InstructionCommand::InitVariable(var) => {
                env.write(var, Number(0));
            }
            InstructionCommand::IncVariable(var) => {
                env.write(var, env.get(&var).clone().succ());
            }
            InstructionCommand::DecVariable(var) => {
                env.write(var, env.get(&var).clone().pred());
            }
            InstructionCommand::CopyVariable(var1, var2) => {
                env.write(var1, env.get(&var2).clone());
            }
        }
    }
    pub fn changed_var(&self) -> Var {
        match self {
            InstructionCommand::InitVariable(var) => var.clone(),
            InstructionCommand::IncVariable(var) => var.clone(),
            InstructionCommand::DecVariable(var) => var.clone(),
            InstructionCommand::CopyVariable(var1, _var2) => var1.clone(),
        }
    }
    pub fn used_var(&self) -> Vec<Var> {
        match self {
            InstructionCommand::InitVariable(var) => vec![var.clone()],
            InstructionCommand::IncVariable(var) => vec![var.clone()],
            InstructionCommand::DecVariable(var) => vec![var.clone()],
            InstructionCommand::CopyVariable(var1, var2) => vec![var1.clone(), var2.clone()],
        }
    }
    pub fn change_var(self, map: &HashMap<Var, Var>) -> Self {
        match self {
            InstructionCommand::InitVariable(var) => {
                InstructionCommand::InitVariable(change_var(var, &map))
            }
            InstructionCommand::IncVariable(var) => {
                InstructionCommand::IncVariable(change_var(var, map))
            }
            InstructionCommand::DecVariable(var) => {
                InstructionCommand::DecVariable(change_var(var, map))
            }
            InstructionCommand::CopyVariable(var1, var2) => {
                InstructionCommand::CopyVariable(change_var(var1, map), change_var(var2, map))
            }
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
    pub fn init(var: Var) -> WhileStatement {
        WhileStatement::Inst(InstructionCommand::InitVariable(var))
    }
    pub fn inc(var: Var) -> WhileStatement {
        WhileStatement::Inst(InstructionCommand::IncVariable(var))
    }
    pub fn dec(var: Var) -> WhileStatement {
        WhileStatement::Inst(InstructionCommand::DecVariable(var))
    }
    pub fn copy(var1: Var, var2: Var) -> WhileStatement {
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
    pub fn change_var(&self, map: &HashMap<Var, Var>) -> WhileStatement {
        match self {
            WhileStatement::Inst(inst) => WhileStatement::Inst(inst.clone().change_var(map)),
            WhileStatement::Cont(ControlCommand::WhileNotZero(var, statements)) => {
                WhileStatement::Cont(ControlCommand::WhileNotZero(
                    change_var(var.clone(), map),
                    statements
                        .into_iter()
                        .map(|statement| statement.change_var(map))
                        .collect(),
                ))
            }
        }
    }
}

impl Display for WhileStatement {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let str: String = match self {
            WhileStatement::Inst(inst) => match inst {
                InstructionCommand::InitVariable(var) => {
                    format!("init {var:?} \n")
                }
                InstructionCommand::IncVariable(var) => {
                    format!("inc {var:?} \n")
                }
                InstructionCommand::DecVariable(var) => {
                    format!("dec {var:?} \n")
                }
                InstructionCommand::CopyVariable(var1, var2) => {
                    format!("copy {var1:?} {var2:?} \n")
                }
            },
            WhileStatement::Cont(ControlCommand::WhileNotZero(var, vec)) => {
                format!("while-is-not-zero {var:?} \n")
                    + &(vec.iter().map(|stm| format!("{stm}")).collect::<String>())
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
                while !(*env.get(var) == Number(0)) {
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
    statements: Vec<WhileStatement>,
}

impl WhileLanguage {
    pub fn changed_var(&self) -> Vec<Var> {
        (&self.statements)
            .iter()
            .flat_map(|statement| statement.changed_var())
            .collect()
    }
    pub fn change_var(&self, map: &HashMap<Var, Var>) -> WhileLanguage {
        (&self.statements)
            .iter()
            .map(|statement| statement.change_var(map))
            .collect::<Vec<_>>()
            .into()
    }
}

impl From<Vec<WhileStatement>> for WhileLanguage {
    fn from(value: Vec<WhileStatement>) -> Self {
        WhileLanguage { statements: value }
    }
}

impl Into<Vec<WhileStatement>> for WhileLanguage {
    fn into(self) -> Vec<WhileStatement> {
        self.statements
    }
}

pub fn eval(prog: &WhileLanguage, mut env: Environment) -> Environment {
    let WhileLanguage { statements } = prog;
    for statement in statements {
        env = eval_statement(statement, env);
    }
    env
}

#[derive(Debug, Clone, PartialEq, Hash)]
pub enum FlatControlCommand {
    WhileNotZero(Var),
    EndWhile,
}

#[derive(Debug, Clone, PartialEq, Hash)]
pub enum FlatWhileStatement {
    Inst(InstructionCommand),
    Cont(FlatControlCommand),
}

impl FlatWhileStatement {
    pub fn init(var: Var) -> FlatWhileStatement {
        FlatWhileStatement::Inst(InstructionCommand::InitVariable(var))
    }
    pub fn inc(var: Var) -> FlatWhileStatement {
        FlatWhileStatement::Inst(InstructionCommand::IncVariable(var))
    }
    pub fn dec(var: Var) -> FlatWhileStatement {
        FlatWhileStatement::Inst(InstructionCommand::DecVariable(var))
    }
    pub fn copy(var1: Var, var2: Var) -> FlatWhileStatement {
        FlatWhileStatement::Inst(InstructionCommand::CopyVariable(var1, var2))
    }
    pub fn while_not_zero(var: Var) -> FlatWhileStatement {
        FlatWhileStatement::Cont(FlatControlCommand::WhileNotZero(var))
    }
    pub fn while_end() -> FlatWhileStatement {
        FlatWhileStatement::Cont(FlatControlCommand::EndWhile)
    }
}

impl Display for FlatWhileStatement {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let str: String = match self {
            FlatWhileStatement::Inst(inst) => match inst {
                InstructionCommand::InitVariable(var) => {
                    format!("init {var:?} \n")
                }
                InstructionCommand::IncVariable(var) => {
                    format!("inc {var:?} \n")
                }
                InstructionCommand::DecVariable(var) => {
                    format!("dec {var:?} \n")
                }
                InstructionCommand::CopyVariable(var1, var2) => {
                    format!("copy {var1:?} {var2:?} \n")
                }
            },
            FlatWhileStatement::Cont(cont) => match cont {
                FlatControlCommand::WhileNotZero(var) => {
                    format!("while-is-not-zero {var:?} \n")
                }
                FlatControlCommand::EndWhile => "end".to_string(),
            },
        };
        write!(f, "{str}")
    }
}

#[derive(Debug, Clone, PartialEq, Hash)]
pub struct FlatWhileLanguage {
    statements: Vec<FlatWhileStatement>,
}

impl FlatWhileLanguage {
    pub fn to_vec(&self) -> Vec<FlatWhileStatement> {
        self.statements.clone()
    }
    pub fn change_var(&self, map: &HashMap<Var, Var>) -> FlatWhileLanguage {
        let vec = self.to_vec();
        vec.into_iter()
            .map(|statement| match statement {
                FlatWhileStatement::Inst(inst) => FlatWhileStatement::Inst(inst.change_var(map)),
                FlatWhileStatement::Cont(command) => {
                    let command = match command {
                        FlatControlCommand::WhileNotZero(var) => {
                            FlatControlCommand::WhileNotZero(change_var(var, map))
                        }
                        FlatControlCommand::EndWhile => FlatControlCommand::EndWhile,
                    };
                    FlatWhileStatement::Cont(command)
                }
            })
            .collect::<Vec<_>>()
            .into()
    }
}

impl Into<Vec<FlatWhileStatement>> for FlatWhileLanguage {
    fn into(self) -> Vec<FlatWhileStatement> {
        self.statements
    }
}

fn flattening(vec: &WhileStatement) -> Vec<FlatWhileStatement> {
    match vec {
        WhileStatement::Inst(inst) => {
            vec![FlatWhileStatement::Inst(inst.clone())]
        }
        WhileStatement::Cont(ControlCommand::WhileNotZero(var, statements)) => {
            let mut vec = vec![FlatWhileStatement::Cont(FlatControlCommand::WhileNotZero(
                var.clone(),
            ))];
            statements.iter().for_each(|statement| {
                vec.extend(flattening(statement));
            });
            vec
        }
    }
}

impl From<&WhileStatement> for FlatWhileLanguage {
    fn from(value: &WhileStatement) -> Self {
        FlatWhileLanguage {
            statements: flattening(&value),
        }
    }
}

impl From<&WhileLanguage> for FlatWhileLanguage {
    fn from(value: &WhileLanguage) -> Self {
        FlatWhileLanguage {
            statements: value
                .statements
                .iter()
                .flat_map(|statement| flattening(statement))
                .collect(),
        }
    }
}

impl From<Vec<FlatWhileStatement>> for FlatWhileLanguage {
    fn from(value: Vec<FlatWhileStatement>) -> Self {
        FlatWhileLanguage { statements: value }
    }
}

fn try_into(vec: &[FlatWhileStatement]) -> Result<Vec<WhileStatement>, ()> {
    eprintln!("{vec:?}");
    let mut now = 0;
    let mut statements = vec![];
    while now < vec.len() {
        let maybe: Result<WhileStatement, ()> = match &vec[now] {
            FlatWhileStatement::Inst(inst) => Ok(WhileStatement::Inst(inst.clone())),
            FlatWhileStatement::Cont(FlatControlCommand::WhileNotZero(var)) => 'a: {
                let mut find_end = now + 1;
                let mut stack = 1;
                while find_end < vec.len() {
                    match vec[find_end] {
                        FlatWhileStatement::Inst(_) => {}
                        FlatWhileStatement::Cont(FlatControlCommand::WhileNotZero(_)) => {
                            stack += 1;
                        }
                        FlatWhileStatement::Cont(FlatControlCommand::EndWhile) => {
                            stack -= 1;
                        }
                    }
                    if stack == 0 {
                        let while_inner = &vec[now + 1..find_end];
                        let vec = try_into(while_inner);
                        let statement =
                            vec.map(|vec| WhileStatement::while_not_zero(var.clone(), vec));
                        now = find_end;
                        break 'a statement;
                    }
                    find_end += 1;
                }
                return Err(());
            }
            FlatWhileStatement::Cont(FlatControlCommand::EndWhile) => {
                return Err(());
            }
        };
        if let Ok(statement) = maybe {
            statements.push(statement);
            now += 1;
        } else {
            return Err(());
        }
    }
    Ok(statements.into())
}

impl TryInto<WhileLanguage> for FlatWhileLanguage {
    type Error = ();
    fn try_into(self) -> Result<WhileLanguage, Self::Error> {
        let statement = try_into(&self.statements);
        Ok(WhileLanguage {
            statements: statement?,
        })
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ProgramProcess {
    prog: FlatWhileLanguage,
    index: usize,
    env: Environment,
}

impl ProgramProcess {
    pub fn new(prog: FlatWhileLanguage, env: Environment) -> Self {
        ProgramProcess {
            prog,
            index: 0,
            env,
        }
    }
    pub fn now_index(&self) -> usize {
        self.index
    }
    pub fn env(&self) -> Environment {
        self.env.clone()
    }
    pub fn code(&self) -> FlatWhileLanguage {
        self.prog.clone()
    }
    pub fn is_terminate(&self) -> bool {
        self.index == self.prog.to_vec().len()
    }
    fn now_statement(&self) -> FlatWhileStatement {
        let ProgramProcess {
            prog,
            index,
            env: _,
        } = self;
        let mut vec = (&prog.statements).clone();
        vec.remove(*index)
    }
    pub fn step(&mut self) {
        if self.is_terminate() {
            return;
        }
        let now_statement = self.now_statement();
        match now_statement {
            FlatWhileStatement::Inst(inst) => {
                inst.eval(&mut self.env);
                self.index += 1;
            }
            FlatWhileStatement::Cont(FlatControlCommand::WhileNotZero(var)) => {
                if self.env.get(&var).is_zero() {
                    let mut stack = 1;
                    loop {
                        self.index += 1;
                        let statement = self.now_statement();
                        if let FlatWhileStatement::Cont(FlatControlCommand::WhileNotZero(_)) =
                            statement
                        {
                            stack += 1;
                        }
                        if let FlatWhileStatement::Cont(FlatControlCommand::EndWhile) = statement {
                            stack -= 1;
                        }
                        if stack == 0 {
                            break;
                        }
                    }
                    self.index += 1;
                } else {
                    self.index += 1;
                }
            }
            FlatWhileStatement::Cont(FlatControlCommand::EndWhile) => {
                let mut stack = 1;
                loop {
                    self.index -= 1;
                    let statement = self.now_statement();
                    if let FlatWhileStatement::Cont(FlatControlCommand::WhileNotZero(_)) = statement
                    {
                        stack -= 1;
                    }
                    if let FlatWhileStatement::Cont(FlatControlCommand::EndWhile) = statement {
                        stack += 1;
                    }
                    if stack == 0 {
                        break;
                    }
                }
            }
        }
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

        let env1: Environment = vec![(Var(0), Number(1))].into();
        let env2: Environment = vec![(Var(0), Number(1))].into();
        assert_eq!(env1, env2);

        let env1: Environment = vec![(Var(0), Number(1)), (Var(0), Number(1))].into();
        let env2: Environment = vec![(Var(0), Number(1))].into();
        assert_eq!(env1, env2);

        let env1: Environment = vec![(Var(0), Number(1)), (Var(1), Number(2))].into();
        let env2: Environment = vec![(Var(1), Number(2)), (Var(0), Number(1))].into();
        assert_eq!(env1, env2);

        let env1: Environment = vec![(Var(0), Number(0))].into();
        let env2: Environment = vec![].into();
        assert_eq!(env1, env2);
    }

    #[test]
    fn eval_test() {
        let env: Environment = Environment::new();
        let prog: WhileLanguage = vec![WhileStatement::inc(Var(0))].into();
        let env_res = eval(&prog, env.clone());
        let env_exp: Environment = vec![(Var(0), Number(1))].into();
        assert_eq!(env_exp, env_res);

        let env: Environment = Environment::new();
        let prog: WhileLanguage = vec![
            WhileStatement::inc(Var(0)),
            WhileStatement::inc(Var(0)),
            WhileStatement::inc(Var(0)),
            WhileStatement::copy(Var(1), Var(0)),
            WhileStatement::copy(Var(0), Var(2)),
        ]
        .into();
        let env_res = eval(&prog, env.clone());
        let env_exp: Environment = vec![(Var(1), Number(3))].into();
        assert_eq!(env_exp, env_res);

        let env: Environment = Environment::new();
        let prog: WhileLanguage = vec![
            WhileStatement::inc(Var(0)),
            WhileStatement::inc(Var(0)),
            WhileStatement::inc(Var(0)),
            WhileStatement::inc(Var(0)),
            WhileStatement::inc(Var(0)),
            WhileStatement::while_not_zero(
                Var(0),
                vec![WhileStatement::dec(Var(0)), WhileStatement::inc(Var(1))],
            ),
        ]
        .into();
        let env_res = eval(&prog, env.clone());
        let env_exp: Environment = vec![(Var(0), Number(0)), (Var(1), Number(5))].into();
        assert_eq!(env_exp, env_res);
    }
    #[test]
    fn flat_to_lang_test() {
        let flat_lang: FlatWhileLanguage = vec![].into();
        let expected: WhileLanguage = vec![].into();
        assert_eq!(flat_lang.try_into(), Ok(expected));

        let flat_lang: FlatWhileLanguage = vec![FlatWhileStatement::inc(Var(0))].into();
        let expected: WhileLanguage = vec![WhileStatement::inc(Var(0))].into();
        assert_eq!(flat_lang.try_into(), Ok(expected));

        let flat_lang: FlatWhileLanguage = vec![
            FlatWhileStatement::while_not_zero(Var(0)),
            FlatWhileStatement::while_end(),
        ]
        .into();
        let expected: WhileLanguage = vec![WhileStatement::while_not_zero(Var(0), vec![])].into();
        assert_eq!(flat_lang.try_into(), Ok(expected));

        let flat_lang: FlatWhileLanguage = vec![
            FlatWhileStatement::while_not_zero(Var(0)),
            FlatWhileStatement::inc(Var(0)),
            FlatWhileStatement::while_not_zero(Var(0)),
            FlatWhileStatement::while_end(),
            FlatWhileStatement::inc(Var(0)),
            FlatWhileStatement::while_end(),
        ]
        .into();
        let expected: WhileLanguage = vec![WhileStatement::while_not_zero(
            Var(0),
            vec![
                WhileStatement::inc(Var(0)),
                WhileStatement::while_not_zero(Var(0), vec![]),
                WhileStatement::inc(Var(0)),
            ],
        )]
        .into();
        assert_eq!(flat_lang.try_into(), Ok(expected));
    }
}
