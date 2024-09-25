use std::collections::HashSet;
use utils::{set::SubSet, variable::Var};

pub trait LambdaExt: Sized {
    type Value: SubSet<Super = Self>;
    type RedexInfo: SubSet<Super = Self>;
    fn free_variables(&self) -> HashSet<Var>;
    fn bound_variables(&self) -> HashSet<Var>;
    fn alpha_conversion_canonical(self, vs: HashSet<Var>) -> Self;
    fn subst(self, x: Var, t: Self) -> Self;

    fn redex_step(r: Self::RedexInfo) -> Self;
    fn step(self) -> Option<Self>;
}

pub trait LambdaContext: LambdaExt {
    type Frame;
    fn decomp(e: Self) -> Option<(Self::Frame, Self)>;
    fn plug(frame: Self::Frame, e: Self) -> Self;
    fn step_state(state: State<Self>) -> Option<State<Self>>;
}

pub struct State<T>
where
    T: LambdaContext,
{
    pub stack: Vec<T::Frame>,
    pub top: T,
}

pub mod ctrl_nat;
pub mod ctrl_ext;
pub mod eff_nat;
pub mod eff_ext;
pub mod grab_nat;
pub mod grab_ext;
pub mod lam_ext;
pub mod lam_nat;
pub mod parse;
pub mod send_nat;
pub mod send_ext;
