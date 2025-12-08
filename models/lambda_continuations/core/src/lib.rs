// ラムダ計算の拡張の実装のためのトレイト
pub mod traits {
    use std::collections::HashSet;

    use utils::variable::Var;

    // 変数周りで実装すべき部分
    pub trait LambdaExt: Sized {
        fn free_variables(&self) -> HashSet<Var>;
        fn bound_variables(&self) -> HashSet<Var>;
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

// core of lambda continuations
pub mod lambda;
// no extension parts
pub mod no_ext;
// abort/control
pub mod ctrl;


// parsing
pub mod parse;

