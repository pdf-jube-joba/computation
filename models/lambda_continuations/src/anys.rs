use std::collections::HashSet;
use utils::variable::Var;

trait LambdaExt {
    fn n_b(var: Var) -> Self;
    fn free_variables(&self) -> HashSet<Var>;
    fn bound_variables(&self) -> HashSet<Var>;
    fn alpha_conversion_canonical(self, vs: HashSet<Var>) -> Self;
    fn subst(self, v: Var, t: Self) -> Self;
}

enum Base<T> {
    Var { var: Var },
    Abs { var: Var, body: T },
    App { e1: T, e2: T },
}

impl<T> LambdaExt for Base<T>
where
    T: LambdaExt,
{
    fn n_b(var: Var) -> Self {
        Base::Var { var }
    }
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

enum Ext<T> {
    Var { var: Var },
    Abs { var: Var, body: T },
    App { e1: T, e2: T },
    Zero,
    Succ { succ: T },
    Pred { pred: T },
    IfZ { cond: T, tcase: T, fcase: T },
    Let { var: Var, bind: T, body: T },
    Rec { fix: Var, var: Var, body: T },
}

impl<T> LambdaExt for Ext<T> {
    fn n_b(var: Var) -> Self {
        Ext::Var { var }
    }
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

enum AbctBase {
    B(Box<Base<AbctBase>>),
    Ab(Box<AbctBase>),
    Ct(Box<AbctBase>),
}

enum AbctExt {
    B(Box<Ext<AbctExt>>),
    Ab(Box<AbctExt>),
    Ct(Box<AbctExt>),
}

enum AbCt<T> {
    B(Box<T>),
    Ab(Box<AbCt<T>>),
    Ct(Box<AbCt<T>>),
}

enum GrDl<T> {
    B(Box<T>),
    Gr(Box<GrDl<T>>),
    Dl(Box<GrDl<T>>),
}
