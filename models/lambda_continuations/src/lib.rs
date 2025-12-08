// ラムダ計算の拡張の実装のためのトレイト
pub mod traits {
    use std::collections::HashSet;

    use utils::variable::Var;

    // 変数周りで実装すべき部分
    pub trait LambdaExt: Sized {
        fn free_variables(&self) -> HashSet<String>;
        fn bound_variables(&self) -> HashSet<String>;
        fn alpha_eq(&self, other: &Self) -> bool;
        fn subst(self, v: Var, t: Self) -> Self;
    }

    // 簡約の定義の実装すべき部分
    pub trait Step: LambdaExt {
        type Value;
        fn is_value(&self) -> Option<Self::Value>;
        fn step(self) -> Option<Self>;
    }

}

pub mod lambda;
pub mod no_ext;
pub mod ctrl;
pub mod parse;
