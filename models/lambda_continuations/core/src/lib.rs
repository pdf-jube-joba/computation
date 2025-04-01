// ラムダ計算の拡張の実装のためのトレイト
pub mod traits {
    use utils::variable::{Var, VarSet};

    // 変数周りで実装すべき部分
    pub trait LambdaExt: Sized {
        fn free_variables(&self) -> VarSet;
        fn bound_variables(&self) -> VarSet;
        fn alpha_eq(&self, other: &Self) -> bool;
        fn subst(self, v: Var, t: Self) -> Self;
    }

    // 簡約の定義の実装すべき部分
    pub trait Step: LambdaExt {
        type Value;
        fn is_value(&self) -> Option<Self::Value>;
        fn step(self) -> Option<Self>;
    }

    // 環境の定義の実装すべき部分
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

    /// associated type を用いた higher kinded type のためののトレイト
    /// LamFamily<HogeStruct>::This == Hoge<T>
    pub trait LamFamily<T> {
        type This;
    }

    pub trait LamFamilySubst<T> {
        // (Base<T>, Var, T) -> T or (Ext<T>, Var, T) -> T
        fn subst_t(self, v: Var, t: T) -> T;
    }
}

pub mod ctrl;
pub mod lambda;
pub mod no_ext;
pub mod parse;

// pub mod ctrl_nat;
// pub mod ctrl_ext;
// pub mod eff_nat;
// pub mod eff_ext;
// pub mod grab_nat;
// pub mod grab_ext;
// pub mod lam_ext;
// pub mod lam_nat;
// pub mod parse;
// pub mod send_nat;
// pub mod send_ext;
// pub mod anys;
