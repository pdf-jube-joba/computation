use utils::variable::Var;

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