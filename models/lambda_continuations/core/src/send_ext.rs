use utils::variable::Var;

pub enum Lam {
    Var(Var),
    Lam(Var, Box<Lam>),
    App(Box<Lam>, Box<Lam>),
    Send(Box<Lam>),
    Run(Box<Lam>, Var, Var, Box<Lam>), // run e1 with x, k => e2
    Zero,
    Succ(Box<Lam>),
    Pred(Box<Lam>),
    IfZ(Box<Lam>, Box<Lam>, Box<Lam>),
    Let(Var, Box<Lam>, Box<Lam>),
    Rec(Var, Var, Box<Lam>),
}

pub enum Value {
    Function(Var, Lam),
}

pub enum RedexInfo {
    // (\x. e) v
    AbsApp {
        x: Var,
        e: Lam,
        v: Value,
    },
    // run v -> v
    RunVal {
        v: Value,
    },
    // run F[send v] with x, k => e
    RunSend {
        cxt: SendPureCxt,
        v: Value,
        x: Var,
        k: Var,
        e: Lam,
    },
}

pub enum SendPureCxt {
    Hole,
    AppR(Box<SendPureCxt>, Lam), // E[[] e]
    AppL(Box<SendPureCxt>, Lam), // E[v []]
}

pub enum SendCxt {
    Hole,
    AppR(Box<SendCxt>, Lam), // E[[] e]
    AppL(Box<SendCxt>, Lam), // E[v []]
    Del(Box<SendCxt>),       // E[send []] ,
}
