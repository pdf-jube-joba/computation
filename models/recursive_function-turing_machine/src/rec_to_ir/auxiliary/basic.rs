use crate::rec_tm_ir::{Function, Stmt};
use crate::rec_to_ir::S;

// Move right until the head reads 'x'. Head stops on 'x'.
// ... |?| A[0] x A[1] x ... A[n - 1] x - ...
// ... ? A[0] x A[1] x ... A[n - 1] |x| - ...
// where
//    - ? in {'-', 'l', 'x'} **this can be 'x', not stop on here**
//    - A[i] consists of {'-', 'l'} and does not contain 'x'
// n == 0 is allowed with doing nothing
pub(crate) fn move_right_till_x_n_times(n: usize) -> Function {
    Function {
        name: format!("move_right_till_x_{n}"),
        params: vec![],
        body: (0..n)
            .map(|i| Stmt::Loop {
                label: format!("until_x_{i}"),
                body: vec![
                    Stmt::Rt,
                    Stmt::IfBreakHead {
                        value: S::X.into(),
                        label: format!("until_x_{i}"),
                    },
                ],
            })
            .collect(),
    }
}

// Move left until the head reads 'x'. Head stops on 'x'.
// ...  x  A[0] x A[1] x ... x A[n - 1] |?| - ...
// ... |x| A[0] x A[1] x ... x A[n - 1]  ? - ...
// where
//    - ? in {'-', 'l', 'x'} **this can be 'x', not stop on here**
//    - A[i] consists of {'-', 'l'} and does not contain 'x'
// n == 0 is allowed with doing nothing
pub(crate) fn move_left_till_x_n_times(n: usize) -> Function {
    Function {
        name: format!("move_left_till_x_{n}"),
        params: vec![],
        body: (0..n)
            .map(|i| Stmt::Loop {
                label: format!("until_x_{i}"),
                body: vec![
                    Stmt::Lt,
                    Stmt::IfBreakHead {
                        value: S::X.into(),
                        label: format!("until_x_{i}"),
                    },
                ],
            })
            .collect(),
    }
}
