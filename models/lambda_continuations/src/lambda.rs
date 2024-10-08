use super::traits::*;
use utils::variable::{Var, VarSet};

pub mod base {
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

    impl<T> From<BaseValue<T>> for Base<T> {
        fn from(value: BaseValue<T>) -> Self {
            match value {
                BaseValue::Fun { var, body } => Base::Lam { var, body },
            }
        }
    }

    impl<T> TryFrom<Base<T>> for BaseValue<T> {
        type Error = ();
        fn try_from(value: Base<T>) -> Result<Self, Self::Error> {
            match value {
                Base::Var { var: _ } => Err(()),
                Base::Lam { var, body } => Ok(BaseValue::Fun { var, body }),
                Base::App { e1: _, e2: _ } => Err(()),
            }
        }
    }

    #[derive(Debug, Clone, PartialEq)]
    pub enum BaseFrame<T> {
        EvalR { left_value: BaseValue<T> },
        EvalL { right_exp: T },
    }

    impl<T> BaseFrame<T>
    where
        T: Clone + From<Base<T>>,
    {
        pub fn decomp<F>(t: Base<T>, f: F) -> Option<(BaseFrame<T>, T)>
        where
            F: Fn(T) -> Option<BaseValue<T>>,
        {
            let Base::App { e1, e2 } = t else {
                return None;
            };
            match f(e1.clone()) {
                Some(value) => Some((BaseFrame::EvalR { left_value: value }, e2)),
                None => Some((BaseFrame::EvalL { right_exp: e2 }, e1)),
            }
        }
        pub fn plug(self, t: T) -> Base<T> {
            match self {
                BaseFrame::EvalR { left_value } => {
                    let value: Base<T> = left_value.into();
                    let value: T = value.into();
                    Base::App { e1: value, e2: t }
                }
                BaseFrame::EvalL { right_exp } => Base::App {
                    e1: right_exp,
                    e2: t,
                },
            }
        }
    }

    impl<T> LamFamilySubst<T> for Base<T>
    where
        T: LambdaExt + From<Base<T>> + Clone + PartialEq,
    {
        /// (Base<T>, Var, T) -> T
        fn subst_t(self, v: Var, t: T) -> T {
            match self {
                Base::Var { var } => {
                    if var == v {
                        t
                    } else {
                        Base::n_v(var).into()
                    }
                }
                Base::Lam { var, body } => {
                    let mut set: VarSet = t.free_variables();
                    let new_var = set.new_var_modify();
                    Base::n_l(
                        new_var.clone(),
                        body.subst(var, Base::n_v(new_var).into()).subst(v, t),
                    )
                    .into()
                }
                Base::App { e1, e2 } => {
                    Base::n_a(e1.subst(v.clone(), t.clone()), e2.subst(v, t)).into()
                }
            }
        }
    }

    impl<T> LambdaExt for Base<T>
    where
        T: LambdaExt + Clone + PartialEq + From<Base<T>>,
    {
        fn free_variables(&self) -> VarSet {
            let mut set = VarSet::default();
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
        fn bound_variables(&self) -> VarSet {
            let mut set = VarSet::default();
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
                ) => {
                    let mut set: VarSet = body1.free_variables();
                    set.union(&body2.free_variables());
                    let new_var = Base::n_v(set.new_var_modify());
                    let body1 = body1.clone().subst(var1.clone(), new_var.clone().into());
                    let body2 = body2.clone().subst(var2.clone(), new_var.into());
                    body1.alpha_eq(&body2)
                }
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
                    let mut set: VarSet = t.free_variables();
                    let new_var = set.new_var_default(var.clone());
                    Base::n_l(
                        new_var.clone(),
                        body.subst(var, Base::n_v(new_var).into())
                            .subst(v, t.into()),
                    )
                }
                Base::App { e1, e2 } => {
                    Base::n_a(e1.subst(v.clone(), t.clone().into()), e2.subst(v, t.into()))
                }
            }
        }
    }

    // higher kinded type のための型
    #[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
    pub struct BaseStruct;
    impl<T> LamFamily<T> for BaseStruct {
        type This = Base<T>;
    }

    #[macro_export]
    macro_rules! bvar {
        ($str: literal) => {
            Base::n_v($str.into()).into()
        };
    }

    #[macro_export]
    macro_rules! blam {
        ($str: literal, $t: expr) => {
            Base::n_l($str.into(), $t).into()
        };
    }

    #[macro_export]
    macro_rules! bapp {
        ($t1: expr, $t2: expr) => {
            Base::n_a($t1, $t2).into()
        };
    }

    pub use {bapp, blam, bvar};
}

pub mod ext {
    use super::*;
    use utils::number::Number;

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

    impl<T> Ext<T> {
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

    impl<T> From<ExtValue<T>> for Ext<T>
    where
        T: From<Ext<T>>,
    {
        fn from(value: ExtValue<T>) -> Self {
            fn num_to_exp<T>(n: Number) -> Ext<T>
            where
                T: From<Ext<T>>,
            {
                if n == 0.into() {
                    Ext::Zero
                } else {
                    let succ: Ext<T> = num_to_exp(n.pred());
                    let succ: T = succ.into();
                    Ext::Succ { succ }
                }
            }
            match value {
                ExtValue::Fun { var, body } => Ext::Lam { var, body },
                ExtValue::Num(n) => num_to_exp(n),
            }
        }
    }

    pub fn num_to_exp<T, F>(n: Number, f: F) -> Ext<T>
    where
        F: Fn(Ext<T>) -> T + Clone,
    {
        if n.is_zero() {
            Ext::Zero
        } else {
            Ext::Succ {
                succ: f(num_to_exp(n.pred(), f.clone())),
            }
        }
    }

    fn exp_to_num<T, F>(t: Ext<T>, f: F) -> Option<Number>
    where
        F: Fn(T) -> Option<Ext<T>>,
    {
        if let Ext::Zero = t {
            Some(0.into())
        } else if let Ext::Succ { succ } = t {
            Some(exp_to_num(f(succ)?, f)?.succ())
        } else {
            None
        }
    }

    pub fn ext_to_ext_value<T, F>(value: Ext<T>, f: F) -> Option<ExtValue<T>>
    where
        F: Fn(T) -> Option<Ext<T>>,
    {
        match value {
            Ext::Lam { var, body } => Some(ExtValue::Fun { var, body }),
            _ => Some(ExtValue::Num(exp_to_num(value, f)?)),
        }
    }

    #[derive(Debug, Clone, PartialEq)]
    pub enum ExtFrame<T> {
        EvalR { left_value: ExtValue<T> },
        EvalL { right_exp: T },
        EvalSucc,
        EvalPred,
        EvalIf { tcase: T, fcase: T },
        EvalLet { var: Var, body: T },
    }

    impl<T> ExtFrame<T>
    where
        T: Clone + From<Ext<T>>,
    {
        pub fn decomp<F>(t: Ext<T>, f: F) -> Option<(ExtFrame<T>, T)>
        where
            F: Fn(T) -> Option<ExtValue<T>> + Clone,
        {
            match t {
                Ext::Var { var: _ } => None,
                Ext::Lam { var: _, body: _ } => None,
                Ext::App { e1, e2 } => match f(e1.clone()) {
                    Some(value) => Some((ExtFrame::EvalR { left_value: value }, e2)),
                    None => Some((ExtFrame::EvalL { right_exp: e2 }, e1)),
                },
                Ext::Zero => None,
                Ext::Succ { succ } => Some((ExtFrame::EvalSucc, succ)),
                Ext::Pred { pred } => Some((ExtFrame::EvalPred, pred)),
                Ext::IfZ { cond, tcase, fcase } => Some((ExtFrame::EvalIf { tcase, fcase }, cond)),
                Ext::Let { var, bind, body } => Some((ExtFrame::EvalLet { var, body }, bind)),
                Ext::Rec {
                    fix: _,
                    var: _,
                    body: _,
                } => None,
            }
        }
        pub fn plug(self, t: T) -> Ext<T> {
            match self {
                ExtFrame::EvalR { left_value } => {
                    let v: Ext<T> = left_value.into();
                    Ext::App {
                        e1: v.into(),
                        e2: t,
                    }
                }
                ExtFrame::EvalL { right_exp } => Ext::App {
                    e1: t,
                    e2: right_exp,
                },
                ExtFrame::EvalSucc => Ext::Succ { succ: t },
                ExtFrame::EvalPred => Ext::Pred { pred: t },
                ExtFrame::EvalIf { tcase, fcase } => Ext::IfZ {
                    cond: t,
                    tcase,
                    fcase,
                },
                ExtFrame::EvalLet { var, body } => Ext::Let { var, bind: t, body },
            }
        }
    }

    impl<T> LambdaExt for Ext<T>
    where
        T: LambdaExt + Clone + PartialEq + From<Ext<T>>,
    {
        fn free_variables(&self) -> VarSet {
            let mut set = VarSet::default();
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
        fn bound_variables(&self) -> VarSet {
            let mut set = VarSet::default();
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
                ) => {
                    let mut set: VarSet = body1.free_variables();
                    set.union(&body2.free_variables());
                    let new_var = Ext::n_v(set.new_var_modify());
                    let body1 = body1.clone().subst(var1.clone(), new_var.clone().into());
                    let body2 = body2.clone().subst(var2.clone(), new_var.into());
                    body1.alpha_eq(&body2)
                }
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
                    body1.alpha_eq(body2)
                        && Ext::Lam {
                            var: var1.clone(),
                            body: bind1.clone(),
                        }
                        .alpha_eq(&Ext::Lam {
                            var: var2.clone(),
                            body: bind2.clone(),
                        })
                    // {
                    //     let mut set: VarSet = bind1.free_variables();
                    //     set.extend(bind2.free_variables());
                    //     let new_var = Ext::n_v(set.new_var_modify());
                    //     let bind1 = bind1.clone().subst(var1.clone(), new_var.clone().into());
                    //     let bind2 = bind2.clone().subst(var2.clone(), new_var.into());
                    //     bind1.alpha_eq(&bind2)
                    // }
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
                    let mut set = body1.free_variables();
                    set.extend(body2.free_variables());
                    let new_var = set.new_var_modify();
                    let new_fix = set.new_var_modify();
                    let body1 = body1
                        .clone()
                        .subst(var1.clone(), Ext::n_v(new_var.clone()).into())
                        .subst(fix1.clone(), Ext::n_v(new_fix.clone()).into());
                    let body2 = body2
                        .clone()
                        .subst(var2.clone(), Ext::n_v(new_var.clone()).into())
                        .subst(fix2.clone(), Ext::n_v(new_fix.clone()).into());
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
                    let mut set: VarSet = t.free_variables();
                    let new_var = set.new_var_modify();
                    Ext::n_l(
                        new_var.clone(),
                        body.subst(var, Ext::n_v(new_var).into()).subst(v, t.into()),
                    )
                }
                Ext::App { e1, e2 } => {
                    Ext::n_a(e1.subst(v.clone(), t.clone().into()), e2.subst(v, t.into()))
                }
                Ext::Zero => Ext::n_z(),
                Ext::Succ { succ } => Ext::n_s(succ.subst(v, t.into())),
                Ext::Pred { pred } => Ext::n_p(pred.subst(v, t.into())),
                Ext::IfZ { cond, tcase, fcase } => Ext::n_i(
                    cond.subst(v.clone(), t.clone().into()),
                    tcase.subst(v.clone(), t.clone().into()),
                    fcase.subst(v, t.into()),
                ),
                Ext::Let { var, bind, body } => {
                    let mut set: VarSet = body.free_variables();
                    let new_var = set.new_var_modify();
                    let bind = bind.subst(v.clone(), t.clone().into());
                    let body = body
                        .subst(var, Ext::n_v(new_var.clone()).into())
                        .subst(v, t.into());
                    Ext::n_d(new_var, bind, body)
                }
                Ext::Rec { fix, var, body } => {
                    let mut set: VarSet = t.free_variables();
                    let new_fix = set.new_var_default(&fix);
                    let new_var = set.new_var_default(&var);
                    let new_body = body
                        .subst(fix.clone(), Ext::n_v(new_fix.clone()).into())
                        .subst(var.clone(), Ext::n_v(new_var.clone()).into())
                        .subst(v, t.into());
                    Ext::n_r(new_fix, new_var, new_body)
                }
            }
        }
    }

    impl<T> LamFamilySubst<T> for Ext<T>
    where
        T: LambdaExt + From<Ext<T>> + Clone,
    {
        /// (Ext<T>, Var, T) -> T
        fn subst_t(self, v: Var, t: T) -> T {
            match self {
                Ext::Var { var } => {
                    if var == v {
                        t
                    } else {
                        Ext::Var { var }.into()
                    }
                }
                Ext::Lam { var, body } => {
                    let mut set: VarSet = t.free_variables();
                    let new_var = set.new_var_modify();
                    Ext::n_l(
                        new_var.clone(),
                        body.subst(var, Ext::n_v(new_var).into()).subst(v, t),
                    )
                    .into()
                }
                Ext::App { e1, e2 } => {
                    Ext::n_a(e1.subst(v.clone(), t.clone()), e2.subst(v, t)).into()
                }
                Ext::Zero => Ext::n_z().into(),
                Ext::Succ { succ } => Ext::n_s(succ.subst(v, t)).into(),
                Ext::Pred { pred } => Ext::n_p(pred.subst(v, t)).into(),
                Ext::IfZ { cond, tcase, fcase } => Ext::n_i(
                    cond.subst(v.clone(), t.clone()),
                    tcase.subst(v.clone(), t.clone()),
                    fcase.subst(v, t),
                )
                .into(),
                Ext::Let { var, bind, body } => {
                    let mut set: VarSet = body.free_variables();
                    let new_var = set.new_var_modify();
                    let bind = bind.subst(v.clone(), t.clone());
                    let body = body
                        .subst(var, Ext::n_v(new_var.clone()).into())
                        .subst(v, t);
                    Ext::n_d(new_var, bind, body).into()
                }
                Ext::Rec { fix, var, body } => {
                    let mut set: VarSet = body.free_variables();
                    let new_fix = set.new_var_default(&fix);
                    let new_var = set.new_var_default(&var);
                    let new_body = body
                        .subst(fix.clone(), Ext::n_v(new_fix.clone()).into())
                        .subst(var.clone(), Ext::n_v(new_var.clone()).into())
                        .subst(v, t);
                    Ext::n_r(new_fix, new_var, new_body).into()
                }
            }
        }
    }

    #[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
    pub struct ExtStruct;
    impl<T> LamFamily<T> for ExtStruct {
        type This = Ext<T>;
    }

    #[macro_export]
    macro_rules! evar {
        ($str: literal) => {
            Ext::n_v($str.into()).into()
        };
    }

    #[macro_export]
    macro_rules! elam {
        ($str: literal, $t: expr) => {
            Ext::n_l($str.into(), $t).into()
        };
    }

    #[macro_export]
    macro_rules! eapp {
        ($t1: expr, $t2: expr) => {
            Ext::n_a($t1, $t2).into()
        };
    }

    #[macro_export]
    macro_rules! ezero {
        () => {
            Ext::Zero.into()
        };
    }

    #[macro_export]
    macro_rules! esucc {
        ($t: expr) => {
            Ext::Succ { succ: $t }.into()
        };
    }

    #[macro_export]
    macro_rules! epred {
        ($t: expr) => {
            Ext::Pred { pred: $t }.into()
        };
    }

    #[macro_export]
    macro_rules! eif {
        ($t: expr, $t1: expr, $t2: expr) => {
            Ext::IfZ {
                cond: $t,
                tcase: $t1,
                fcase: $t2,
            }
            .into()
        };
    }

    #[macro_export]
    macro_rules! elet {
        ($t: literal, $t1: expr, $t2: expr) => {
            Ext::Let {
                var: $t.into(),
                bind: $t1,
                body: $t2,
            }
            .into()
        };
    }

    #[macro_export]
    macro_rules! erec {
        ($t: literal, $t1: literal, $t2: expr) => {
            Ext::Rec {
                fix: $t.into(),
                var: $t1.into(),
                body: $t2,
            }
            .into()
        };
    }

    pub use {eapp, eif, elam, elet, epred, erec, esucc, evar, ezero};
}
