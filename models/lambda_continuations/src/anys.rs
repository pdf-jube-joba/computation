use std::collections::HashSet;
use utils::{number::Number, set::SubSet, variable::Var};

pub mod traits {
    use super::*;
    // ラムダ計算の変数周りの trait
    pub trait LambdaExt: Sized {
        fn free_variables(&self) -> HashSet<Var>;
        fn bound_variables(&self) -> HashSet<Var>;
        fn alpha_eq(&self, other: &Self) -> bool;
        fn subst(self, v: Var, t: Self) -> Self;
    }

    pub trait Step: LambdaExt {
        type Value: SubSet<Super = Self>;
        fn step(self) -> Option<Result<Self::Value, Self>>;
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

    pub trait LamFamily<T> {
        type This: LambdaExt;
    }
}

pub mod ext {
    use utils::variable::VarSet;

    use super::traits::*;
    use super::*;

    #[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
    pub enum Base<T> {
        Var { var: Var },
        Lam { var: Var, body: T },
        App { e1: T, e2: T },
    }

    impl<T> Base<T> {
        pub fn n_v(var: Var) -> Self {
            Base::Var { var }
        }
        pub fn n_l(var: Var, body: T) -> Self {
            Base::Lam { var, body }
        }
        pub fn n_a(e1: T, e2: T) -> Self {
            Base::App { e1, e2 }
        }
    }

    #[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
    pub enum BaseValue<T> {
        Fun { var: Var, body: T },
    }

    // BaseValue<T> \subset Base<T> \subset T
    impl<T> SubSet for BaseValue<T>
    where
        T: Clone,
    {
        type Super = Base<T>;
        fn from_super(s: &Self::Super) -> Option<Self> {
            match s {
                Base::Var { var: _ } => None,
                Base::Lam { var, body } => Some(BaseValue::Fun {
                    var: var.clone(),
                    body: body.clone(),
                }),
                Base::App { e1: _, e2: _ } => None,
            }
        }
        fn into_super(self) -> Self::Super {
            match self {
                BaseValue::Fun { var, body } => Base::Lam { var, body },
            }
        }
    }

    impl<T> LambdaExt for Base<T>
    where
        T: LambdaExt + Clone + PartialEq,
        Base<T>: SubSet<Super = T>,
    {
        fn free_variables(&self) -> HashSet<Var> {
            let mut set = HashSet::new();
            match self {
                Base::Var { var } => {
                    set.insert(var.clone());
                }
                Base::Lam { var, body } => {
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
            let mut set = HashSet::new();
            match self {
                Base::Var { var: _ } => {}
                Base::Lam { var, body } => {
                    set.extend(body.bound_variables());
                    set.insert(var.clone());
                }
                Base::App { e1, e2 } => {
                    set.extend(e1.bound_variables());
                    set.extend(e2.bound_variables());
                }
            }
            set
        }
        fn alpha_eq(&self, other: &Self) -> bool {
            match (self, other) {
                (Base::Var { var: var1 }, Base::Var { var: var2 }) => var1 == var2,
                (Base::App { e1: m1, e2: m2 }, Base::App { e1: n1, e2: n2 }) => {
                    m1.alpha_eq(n1) && m2.alpha_eq(n2)
                }
                (
                    Base::Lam {
                        var: var1,
                        body: body1,
                    },
                    Base::Lam {
                        var: var2,
                        body: body2,
                    },
                ) => body1.alpha_eq(
                    &body2
                        .clone()
                        .subst(var2.clone(), Base::n_v(var1.clone()).into_super()),
                ),
                _ => false,
            }
        }
        fn subst(self, v: Var, t: Self) -> Self {
            match self {
                Base::Var { var } => {
                    if var == v {
                        t
                    } else {
                        Base::Var { var }
                    }
                }
                Base::Lam { var, body } => {
                    let mut set: VarSet = t.free_variables().into();
                    let new_var = set.new_var_modify();
                    Base::n_l(
                        new_var.clone(),
                        body.subst(var, Base::n_v(new_var).into_super())
                            .subst(v, t.into_super()),
                    )
                }
                Base::App { e1, e2 } => Base::n_a(
                    e1.subst(v.clone(), t.clone().into_super()),
                    e2.subst(v, t.into_super()),
                ),
            }
        }
    }

    #[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
    pub struct BaseStruct;
    impl<T> LamFamily<T> for BaseStruct
    where
        T: LambdaExt + Clone + PartialEq,
        Base<T>: SubSet<Super = T>,
    {
        type This = Base<T>;
    }

    #[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
    pub enum Ext<T> {
        Var { var: Var },
        Lam { var: Var, body: T },
        App { e1: T, e2: T },
        Zero,
        Succ { succ: T },
        Pred { pred: T },
        IfZ { cond: T, tcase: T, fcase: T },
        Let { var: Var, bind: T, body: T },
        Rec { fix: Var, var: Var, body: T },
    }

    impl<T> Ext<T>
    where
        T: Clone + PartialEq,
    {
        pub fn n_v(var: Var) -> Self {
            Ext::Var { var }
        }
        pub fn n_l(var: Var, body: T) -> Self {
            Ext::Lam { var, body }
        }
        pub fn n_a(e1: T, e2: T) -> Self {
            Ext::App { e1, e2 }
        }
        pub fn n_z() -> Self {
            Ext::Zero
        }
        pub fn n_s(succ: T) -> Self {
            Ext::Succ { succ }
        }
        pub fn n_p(pred: T) -> Self {
            Ext::Pred { pred }
        }
        pub fn n_i(cond: T, tcase: T, fcase: T) -> Self {
            Ext::IfZ { cond, tcase, fcase }
        }
        pub fn n_d(var: Var, bind: T, body: T) -> Self {
            Ext::Let { var, bind, body }
        }
        pub fn n_r(fix: Var, var: Var, body: T) -> Self {
            Ext::Rec { fix, var, body }
        }
    }

    #[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
    pub enum ExtValue<T> {
        Fun { var: Var, body: T },
        Num(Number),
    }

    fn num_to_exp<T>(n: Number) -> Ext<T>
    where
        Ext<T>: SubSet<Super = T>,
    {
        if n == 0.into() {
            Ext::Zero
        } else {
            let succ: Ext<T> = num_to_exp(n.pred());
            let succ: T = succ.into_super();
            Ext::Succ { succ }
        }
    }

    impl<T> SubSet for ExtValue<T>
    where
        T: LambdaExt + Clone + PartialEq,
        Ext<T>: SubSet<Super = T>,
    {
        type Super = Ext<T>;
        fn from_super(s: &Self::Super) -> Option<Self> {
            match s {
                Ext::Lam { var, body } => Some(ExtValue::Fun {
                    var: var.clone(),
                    body: body.clone(),
                }),
                Ext::Zero => Some(ExtValue::Num(0.into())),
                Ext::Succ { succ } => {
                    let ext_t = Ext::from_super(succ)?;
                    if let ExtValue::Num(n) = ExtValue::from_super(&ext_t)? {
                        Some(ExtValue::Num(n.succ()))
                    } else {
                        None
                    }
                }
                _ => None,
            }
        }
        fn into_super(self) -> Self::Super {
            match self {
                ExtValue::Fun { var, body } => Ext::Lam { var, body },
                ExtValue::Num(n) => num_to_exp(n),
            }
        }
    }

    impl<T> LambdaExt for Ext<T>
    where
        T: LambdaExt + Clone + PartialEq,
        Ext<T>: SubSet<Super = T>,
    {
        fn free_variables(&self) -> HashSet<Var> {
            let mut set = HashSet::new();
            match self {
                Ext::Var { var } => {
                    set.insert(var.clone());
                }
                Ext::Lam { var, body } => {
                    set.extend(body.free_variables());
                    set.remove(var);
                }
                Ext::App { e1, e2 } => {
                    set.extend(e1.free_variables());
                    set.extend(e2.free_variables());
                }
                Ext::Zero => {}
                Ext::Succ { succ } => set.extend(succ.free_variables()),
                Ext::Pred { pred } => {
                    set.extend(pred.free_variables());
                }
                Ext::IfZ { cond, tcase, fcase } => {
                    set.extend(cond.free_variables());
                    set.extend(tcase.free_variables());
                    set.extend(fcase.free_variables());
                }
                Ext::Let { var, bind, body } => {
                    set.extend(body.free_variables());
                    set.remove(var);
                    set.extend(bind.free_variables());
                }
                Ext::Rec { fix, var, body } => {
                    set.extend(body.free_variables());
                    set.remove(fix);
                    set.remove(var);
                }
            }
            set
        }
        fn bound_variables(&self) -> HashSet<Var> {
            let mut set = HashSet::new();
            match self {
                Ext::Var { var: _ } => {}
                Ext::Lam { var, body } => {
                    set.extend(body.bound_variables());
                    set.insert(var.clone());
                }
                Ext::App { e1, e2 } => {
                    set.extend(e1.bound_variables());
                    set.extend(e2.bound_variables());
                }
                Ext::Zero => {}
                Ext::Succ { succ } => set.extend(succ.bound_variables()),
                Ext::Pred { pred } => {
                    set.extend(pred.bound_variables());
                }
                Ext::IfZ { cond, tcase, fcase } => {
                    set.extend(cond.bound_variables());
                    set.extend(tcase.bound_variables());
                    set.extend(fcase.bound_variables());
                }
                Ext::Let { var, bind, body } => {
                    set.extend(body.bound_variables());
                    set.insert(var.clone());
                    set.extend(bind.bound_variables());
                }
                Ext::Rec { fix, var, body } => {
                    set.extend(body.bound_variables());
                    set.insert(fix.clone());
                    set.insert(var.clone());
                }
            }
            set
        }
        fn alpha_eq(&self, other: &Self) -> bool {
            match (self, other) {
                (Ext::Var { var: var1 }, Ext::Var { var: var2 }) => var1 == var2,
                (Ext::App { e1: m1, e2: m2 }, Ext::App { e1: n1, e2: n2 }) => {
                    m1.alpha_eq(n1) && m2.alpha_eq(n2)
                }
                (
                    Ext::Lam {
                        var: var1,
                        body: body1,
                    },
                    Ext::Lam {
                        var: var2,
                        body: body2,
                    },
                ) => body1.alpha_eq(
                    &body2
                        .clone()
                        .subst(var2.clone(), Ext::n_v(var1.clone()).into_super()),
                ),
                (Ext::Zero, Ext::Zero) => true,
                (Ext::Succ { succ: succ1 }, Ext::Succ { succ: succ2 }) => succ1.alpha_eq(succ2),
                (Ext::Pred { pred: pred1 }, Ext::Pred { pred: pred2 }) => pred1.alpha_eq(pred2),
                (
                    Ext::IfZ {
                        cond: cond1,
                        tcase: tcase1,
                        fcase: fcase1,
                    },
                    Ext::IfZ {
                        cond: cond2,
                        tcase: tcase2,
                        fcase: fcase2,
                    },
                ) => cond1.alpha_eq(cond2) && tcase1.alpha_eq(tcase2) && fcase1.alpha_eq(fcase2),
                (
                    Ext::Let {
                        var: var1,
                        bind: bind1,
                        body: body1,
                    },
                    Ext::Let {
                        var: var2,
                        bind: bind2,
                        body: body2,
                    },
                ) => {
                    body1.alpha_eq(body2) && {
                        bind1.alpha_eq(
                            &bind2
                                .clone()
                                .subst(var2.clone(), Ext::n_v(var1.clone()).into_super()),
                        )
                    }
                }
                (
                    Ext::Rec {
                        fix: fix1,
                        var: var1,
                        body: body1,
                    },
                    Ext::Rec {
                        fix: fix2,
                        var: var2,
                        body: body2,
                    },
                ) => {
                    let body2 = body2
                        .clone()
                        .subst(fix2.clone(), Ext::n_v(fix1.clone()).into_super())
                        .subst(var2.clone(), Ext::n_v(var1.clone()).into_super());
                    body1.alpha_eq(&body2)
                }
                _ => false,
            }
        }
        fn subst(self, v: Var, t: Self) -> Self {
            match self {
                Ext::Var { var } => {
                    if var == v {
                        t
                    } else {
                        Ext::Var { var }
                    }
                }
                Ext::Lam { var, body } => {
                    let mut set: VarSet = t.free_variables().into();
                    let new_var = set.new_var_modify();
                    Ext::n_l(
                        new_var.clone(),
                        body.subst(var, Ext::n_v(new_var).into_super())
                            .subst(v, t.into_super()),
                    )
                }
                Ext::App { e1, e2 } => Ext::n_a(
                    e1.subst(v.clone(), t.clone().into_super()),
                    e2.subst(v, t.into_super()),
                ),
                Ext::Zero => Ext::n_z(),
                Ext::Succ { succ } => Ext::n_s(succ.subst(v, t.into_super())),
                Ext::Pred { pred } => Ext::n_p(pred.subst(v, t.into_super())),
                Ext::IfZ { cond, tcase, fcase } => Ext::n_i(
                    cond.subst(v.clone(), t.clone().into_super()),
                    tcase.subst(v.clone(), t.clone().into_super()),
                    fcase.subst(v, t.into_super()),
                ),
                Ext::Let { var, bind, body } => {
                    let mut set: VarSet = body.free_variables().into();
                    let new_var = set.new_var_modify();
                    let bind = bind.subst(v.clone(), t.clone().into_super());
                    let body = body
                        .subst(var, Ext::n_v(new_var.clone()).into_super())
                        .subst(v, t.into_super());
                    Ext::n_d(new_var, bind, body)
                }
                Ext::Rec { fix, var, body } => {
                    let mut set: VarSet = body.free_variables().into();
                    let new_fix = set.new_var_default(&fix);
                    let new_var = set.new_var_default(&var);
                    let new_body = body
                        .subst(fix.clone(), Ext::n_v(new_fix.clone()).into_super())
                        .subst(var.clone(), Ext::n_v(new_var.clone()).into_super())
                        .subst(v, t.into_super());
                    Ext::n_r(new_fix, new_var, new_body)
                }
            }
        }
    }

    #[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
    pub struct ExtStruct;
    impl<T> LamFamily<T> for ExtStruct
    where
        T: LambdaExt + Clone + PartialEq,
        Ext<T>: SubSet<Super = T>,
    {
        type This = Ext<T>;
    }
}

mod lambda {
    use self::ext::{Base, BaseStruct};

    use super::{traits::*, *};

    #[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
    pub enum LamStruct<E>
    where
        E: LamFamily<LamStruct<E>>,
    {
        B(Box<E::This>),
    }

    impl SubSet for Base<LamStruct<BaseStruct>> {
        type Super = LamStruct<BaseStruct>;
        fn from_super(s: &Self::Super) -> Option<Self> {
            todo!()
        }
        fn into_super(self) -> Self::Super {
            todo!()
        }
    }

    impl<E> LambdaExt for LamStruct<E>
    where
        E: LamFamily<LamStruct<E>>,
    {
        fn free_variables(&self) -> HashSet<Var> {
            let LamStruct::B(b) = self;
            b.free_variables()
        }
        fn bound_variables(&self) -> HashSet<Var> {
            let LamStruct::B(b) = self;
            b.bound_variables()
        }
        fn alpha_eq(&self, other: &Self) -> bool {
            let LamStruct::B(b1) = self;
            let LamStruct::B(b2) = other;
            b1.alpha_eq(b2)
        }
        fn subst(self, v: Var, t: Self) -> Self {
            let LamStruct::B(b) = self;
            let LamStruct::B(t) = t;
            LamStruct::B(Box::new(b.subst(v, *t)))
        }
    }

    type LamBase = LamStruct<BaseStruct>;
    fn f() {
        let b: Base<LamBase> = Base::n_v(0.into());
        LamStruct::B(Box::new(b));
    }
}

mod abct {
    use super::{ext::*, traits::*, *};
    pub enum AbCtStruct<E>
    where
        E: LamFamily<AbCtStruct<E>>,
    {
        B(Box<E::This>),
        Ab(Box<AbCtStruct<E>>),
        Ct(Box<AbCtStruct<E>>),
    }

    impl<E> LambdaExt for AbCtStruct<E>
    where
        E: LamFamily<AbCtStruct<E>>,
    {
        fn free_variables(&self) -> HashSet<Var> {
            todo!()
        }
        fn bound_variables(&self) -> HashSet<Var> {
            todo!()
        }
        fn alpha_eq(&self, other: &Self) -> bool {
            todo!()
        }
        fn subst(self, v: Var, t: Self) -> Self {
            todo!()
        }
    }
}

mod grdl {
    use super::{ext::*, traits::*, *};
    pub enum GrDlStruct<E>
    where
        E: LamFamily<GrDlStruct<E>>,
    {
        B(Box<E::This>),
        Gr(Box<GrDlStruct<E>>),
        Dl(Box<GrDlStruct<E>>),
    }

    impl<E> LambdaExt for GrDlStruct<E>
    where
        E: LamFamily<GrDlStruct<E>>,
    {
        fn free_variables(&self) -> HashSet<Var> {
            todo!()
        }
        fn bound_variables(&self) -> HashSet<Var> {
            todo!()
        }
        fn alpha_eq(&self, other: &Self) -> bool {
            todo!()
        }
        fn subst(self, v: Var, t: Self) -> Self {
            todo!()
        }
    }
}
