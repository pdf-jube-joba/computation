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

pub(crate) fn call_r(n: usize) -> Stmt {
    Stmt::Call {
        name: format!("move_right_till_x_{n}"),
        args: vec![],
    }
}

pub(crate) fn call_l(n: usize) -> Stmt {
    Stmt::Call {
        name: format!("move_left_till_x_{n}"),
        args: vec![],
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

// Concat 2 tuples
// ... |x| A x B x - ...
// ... |x| A B x - ...
// where
//   - A, B consists of {'-', 'l'} and does not contain 'x'
pub(crate) fn concat() -> Function {
    Function {
        name: "concat".to_string(),
        params: vec![],
        body: vec![
            Stmt::Call {
                name: "move_right_till_x_2_times".to_string(),
                args: vec![],
            },
            // "swap" (head == 'x')
            Stmt::ConstAssign("put".to_string(), S::X.into()),
            Stmt::StorConst(S::B.into()),
            Stmt::Loop {
                label: "loop".to_string(),
                body: vec![
                    Stmt::Lt,
                    // swap
                    Stmt::Read("tmp".to_string()),
                    Stmt::Stor("put".to_string()),
                    Stmt::Assign("put".to_string(), "tmp".to_string()),
                    // if put == 'x' break
                    Stmt::IfBreak {
                        var: "put".to_string(),
                        value: S::X.into(),
                        label: "loop".to_string(),
                    },
                ],
            },
        ],
    }
}
