use utils::variable::Var;

pub enum LamSendRun {
    Var(Var),
    Lam(Var, Box<LamSendRun>),
    App(Box<LamSendRun>, Box<LamSendRun>),
    Send(Box<LamSendRun>),
    Run(Box<LamSendRun>, Var, Box<LamSendRun>),
}

pub enum SendPureCxt {
    Hole,
    AppR(Box<SendPureCxt>, LamSendRun), // E[[] e]
    AppL(Box<SendPureCxt>, LamSendRun), // E[v []]
}

pub enum SendCxt {
    Hole,
    AppR(Box<SendCxt>, LamSendRun), // E[[] e]
    AppL(Box<SendCxt>, LamSendRun), // E[v []]
    Del(Box<SendCxt>),              // E[send []] ,
}
