use crate::lambda::{base::Base, ext::Ext, LamFamily};
use utils::variable::Var;

enum AbCt<T>
where
    T: LamFamily<AbCt<T>>,
{
    Base(Box<T::This>),
    Abort(Box<AbCt<T>>),
    Control(Box<AbCt<T>>),
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lambda::{base, ext};

    #[test]
    fn t() {
        let v: Var = "test".into();
        let l: AbCt<base::BaseStruct> = AbCt::Base(Box::new(base::Base::n_v(v)));
    }
}
