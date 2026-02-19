use crate::rec_tm_ir::{Block, Condition, Function, LValue, RValue, Stmt};
use crate::rec_to_ir::S;
use crate::{assign, cond, lv, rv};

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
        blocks: (0..n)
            .map(|i| Block {
                label: format!("until_x_{i}"),
                body: vec![
                    Stmt::Rt,
                    Stmt::Break {
                        cond: cond!(rv!(@), rv!(const S::X)),
                    },
                    Stmt::Continue { cond: None },
                ],
            })
            .collect(),
    }
}

pub(crate) fn call_r(n: usize) -> Stmt {
    Stmt::Call {
        name: format!("move_right_till_x_{n}"),
    }
}

pub(crate) fn call_l(n: usize) -> Stmt {
    Stmt::Call {
        name: format!("move_left_till_x_{n}"),
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
        blocks: (0..n)
            .map(|i| Block {
                label: format!("until_x_{i}"),
                body: vec![
                    Stmt::Lt,
                    Stmt::Break {
                        cond: cond!(rv!(@), rv!(const S::X)),
                    },
                    Stmt::Continue { cond: None },
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
        blocks: vec![
            Block {
                label: "initially".to_string(),
                body: vec![
                    call_r(2),
                    // "swap" (head == 'x')
                    // Stmt::AssignConst("put".to_string(), S::X.into()),
                    assign!(lv!("put"), rv!(const S::X)),
                    // Stmt::StorConst(S::B.into()),
                    assign!(lv!(@), rv!(const S::B)),
                ],
            },
            Block {
                label: "loop".to_string(),
                body: vec![
                    Stmt::Lt,
                    // swap
                    assign!(lv!("tmp"), rv!(@)),
                    assign!(lv!(@), rv!("put")),
                    assign!(lv!("put"), rv!("tmp")),
                    // if put == 'x' break
                    Stmt::Return {
                        cond: cond!(rv!("put"), rv!(const S::X)),
                    },
                ],
            },
        ],
    }
}
