use std::collections::HashSet;
use utils::{number::Number, set::SubSet, variable::Var};

pub mod traits {
    use super::*;
    // ラムダ計算の変数周りの trait
    pub trait LambdaExt: Sized {
        fn free_variables(&self) -> HashSet<Var>;
        fn bound_variables(&self) -> HashSet<Var>;
        fn alpha_conversion_canonical(self, vs: HashSet<Var>) -> Self;
        fn subst(self, v: Var, t: Self) -> Self;
    }

    pub trait Step: LambdaExt {
        type Value: SubSet<Super = Self>;
        fn step(self) -> Option<Self>;
    }

    pub trait CallState: LambdaExt {
        type Frame;
        fn step_state(state: State<Self>) -> Option<State<Self>>;
    }

    pub struct State<T>
    where
        T: CallState,
    {
        pub call_stack: Vec<T::Frame>,
        pub top: T,
    }
}

mod ext {
    use super::traits::*;
    use super::*;

    #[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
    pub enum Base<T> {
        Var { var: Var },
        Abs { var: Var, body: T },
        App { e1: T, e2: T },
    }

    #[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
    pub enum BaseValue<T> {
        Fun { var: Var, body: T },
    }

    impl<T> SubSet for BaseValue<T>
    where
        T: Clone,
    {
        type Super = Base<T>;
        fn from_super(s: &Self::Super) -> Option<Self> {
            match s {
                Base::Var { var: _ } => None,
                Base::Abs { var, body } => Some(BaseValue::Fun {
                    var: var.clone(),
                    body: body.clone(),
                }),
                Base::App { e1: _, e2: _ } => None,
            }
        }
        fn into_super(self) -> Self::Super {
            match self {
                BaseValue::Fun { var, body } => Base::Abs { var, body },
            }
        }
    }

    impl<T> LambdaExt for Base<T>
    where
        T: LambdaExt + Clone,
        Base<T>: SubSet<Super = T>,
    {
        fn free_variables(&self) -> HashSet<Var> {
            let mut set = HashSet::new();
            match self {
                Base::Var { var } => {
                    set.insert(var.clone());
                }
                Base::Abs { var, body } => {
                    set.extend(body.free_variables());
                    set.remove(var);
                }
                Base::App { e1, e2 } => {
                    set.extend(e1.free_variables());
                    set.extend(e2.free_variables());
                }
            }
            set
        }
        fn bound_variables(&self) -> HashSet<Var> {
            todo!()
        }
        fn alpha_conversion_canonical(self, vs: HashSet<Var>) -> Self {
            todo!()
        }
        fn subst(self, v: Var, t: Self) -> Self {
            todo!()
        }
    }

    #[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
    pub enum Ext<T> {
        Base { base: Base<T> },
        Zero,
        Succ { succ: T },
        Pred { pred: T },
        IfZ { cond: T, tcase: T, fcase: T },
        Let { var: Var, bind: T, body: T },
        Rec { fix: Var, var: Var, body: T },
    }

    #[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
    pub enum ExtValue<T> {
        Fun { var: Var, body: T },
        Num(Number),
    }

    impl<T> SubSet for ExtValue<T>
    where
        T: Clone,
        Ext<T>: SubSet<Super = T>,
    {
        type Super = Ext<T>;
        fn from_super(s: &Self::Super) -> Option<Self> {
            match s {
                Ext::Zero => Some(ExtValue::Num(0.into())),
                Ext::Succ { succ } => {
                    let ext_t = Ext::from_super(succ)?;
                    if let ExtValue::Num(n) = ExtValue::from_super(&ext_t)? {
                        Some(ExtValue::Num(n.succ()))
                    } else {
                        None
                    }
                }
                Ext::Base {
                    base: Base::Abs { var, body },
                } => Some(ExtValue::Fun {
                    var: var.clone(),
                    body: body.clone(),
                }),
                _ => todo!(),
            }
        }
        fn into_super(self) -> Self::Super {
            todo!()
        }
    }

    impl<T> LambdaExt for Ext<T> {
        fn free_variables(&self) -> HashSet<Var> {
            todo!()
        }
        fn bound_variables(&self) -> HashSet<Var> {
            todo!()
        }
        fn alpha_conversion_canonical(self, vs: HashSet<Var>) -> Self {
            todo!()
        }
        fn subst(self, v: Var, t: Self) -> Self {
            todo!()
        }
    }
}

mod lambda {
    use super::{ext::*, traits::*, *};

    pub struct BaseStruct;
    pub struct ExtStruct;

    pub trait Lam<T> {
        type This: LambdaExt;
    }

    impl<T> Lam<T> for BaseStruct
    where
        T: LambdaExt + Clone,
        Base<T>: SubSet<Super = T>,
    {
        type This = Base<T>;
    }

    impl<T> Lam<T> for ExtStruct
    where
        T: LambdaExt,
        Ext<T>: SubSet<Super = T>,
    {
        type This = Ext<T>;
    }

    pub enum LamStruct<E>
    where
        E: Lam<LamStruct<E>>,
    {
        B(Box<E::This>),
    }

    impl<E> LambdaExt for LamStruct<E>
    where
        E: Lam<LamStruct<E>>,
    {
        fn free_variables(&self) -> HashSet<Var> {
            todo!()
        }
        fn bound_variables(&self) -> HashSet<Var> {
            todo!()
        }
        fn alpha_conversion_canonical(self, vs: HashSet<Var>) -> Self {
            todo!()
        }
        fn subst(self, v: Var, t: Self) -> Self {
            todo!()
        }
    }
}

mod abct {
    use super::{ext::*, traits::*, *};

    pub struct BaseStruct;
    pub struct ExtStruct;

    pub trait AbCt<T> {
        type This: LambdaExt;
    }

    impl<T> AbCt<T> for BaseStruct
    where
        T: LambdaExt + Clone,
        Base<T>: SubSet<Super = T>,
    {
        type This = Base<T>;
    }

    impl<T> AbCt<T> for ExtStruct
    where
        T: LambdaExt,
    {
        type This = Ext<T>;
    }

    pub enum AbCtStruct<E>
    where
        E: AbCt<AbCtStruct<E>>,
    {
        B(Box<E::This>),
        Ab(Box<AbCtStruct<E>>),
        Ct(Box<AbCtStruct<E>>),
    }

    impl<E> LambdaExt for AbCtStruct<E>
    where
        E: AbCt<AbCtStruct<E>>,
    {
        fn free_variables(&self) -> HashSet<Var> {
            todo!()
        }
        fn bound_variables(&self) -> HashSet<Var> {
            todo!()
        }
        fn alpha_conversion_canonical(self, vs: HashSet<Var>) -> Self {
            todo!()
        }
        fn subst(self, v: Var, t: Self) -> Self {
            todo!()
        }
    }
}

mod grdl {
    use super::{ext::*, traits::*, *};

    pub struct BaseStruct;
    pub struct ExtStruct;

    pub trait GrDl<T> {
        type This: LambdaExt;
    }

    impl<T> GrDl<T> for BaseStruct
    where
        T: LambdaExt + Clone,
        Base<T>: SubSet<Super = T>,
    {
        type This = Base<T>;
    }

    impl<T> GrDl<T> for ExtStruct
    where
        T: LambdaExt,
    {
        type This = Ext<T>;
    }

    pub enum GrDlStruct<E>
    where
        E: GrDl<GrDlStruct<E>>,
    {
        B(Box<E::This>),
        Gr(Box<GrDlStruct<E>>),
        Dl(Box<GrDlStruct<E>>),
    }

    impl<E> LambdaExt for GrDlStruct<E>
    where
        E: GrDl<GrDlStruct<E>>,
    {
        fn free_variables(&self) -> HashSet<Var> {
            todo!()
        }
        fn bound_variables(&self) -> HashSet<Var> {
            todo!()
        }
        fn alpha_conversion_canonical(self, vs: HashSet<Var>) -> Self {
            todo!()
        }
        fn subst(self, v: Var, t: Self) -> Self {
            todo!()
        }
    }
}
