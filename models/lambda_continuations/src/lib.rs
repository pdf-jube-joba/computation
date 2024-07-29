use utils::variable::Var;
pub mod lam;
pub mod grab;

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

pub enum LamEffect {
    Var(Var),
    Lam(Var, Box<LamEffect>),
    App(Box<LamEffect>, Box<LamEffect>),
    Op(String, Box<LamEffect>),
    Handle(Box<LamEffect>, Handlers),
}

pub enum EffPureCxt {
    Hole,
    AppR(Box<EffPureCxt>, LamEffect), // E[[] e]
    AppL(Box<EffPureCxt>, LamEffect), // E[v []]
}

pub enum EffCxt {
    Hole,
    AppR(Box<EffCxt>, LamEffect), // E[[] e]
    AppL(Box<EffCxt>, LamEffect), // E[v []]
    Del(Box<EffCxt>),             // E[op []] ,
}

pub struct Handlers(Vec<(String, Var, Var, Box<LamEffect>)>);
