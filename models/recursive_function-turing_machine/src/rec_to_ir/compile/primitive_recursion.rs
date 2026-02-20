use crate::rec_tm_ir::{Block, Function, Stmt};
use crate::rec_to_ir::S;
use crate::rec_to_ir::auxiliary::basic::{call_l, call_r};
use crate::{assign, cond, lv, rv};

// ... ? |x| - l(n) A x - ...
// ... ?  x l |x| - l(n) A x - ...
pub(crate) fn insert_sig() -> Function {
    Function {
        name: "insert_sig".to_string(),
        blocks: vec![
            Block {
                label: "initially".to_string(),
                body: vec![
                    assign!(lv!("tmp1"), rv!(const S::L)),
                    assign!(lv!("tmp2"), rv!(const S::X)),
                    Stmt::Rt,
                ],
            },
            Block {
                label: "main_loop".to_string(),
                body: vec![
                    Stmt::Break {
                        cond: cond!(rv!(@), rv!(const S::X)),
                    },
                    assign!(lv!(@), rv!("tmp1")),
                    assign!(lv!("tmp1"), rv!("tmp2")),
                    Stmt::Rt,
                    assign!(lv!("tmp2"), rv!(@)),
                ],
            },
            Block {
                label: "finally".to_string(),
                body: vec![
                    assign!(lv!(@), rv!("tmp1")),
                    Stmt::Rt,
                    assign!(lv!(@), rv!("tmp2")),
                    call_l(1),
                ],
            },
        ],
    }
}

// |x| -   l(n)  A x - ...
// |x| - l(n-1) A x -
// A is empty or start with '-'
// if n == 0 is given => return x A x
pub(crate) fn pred_tuple() -> Function {
    Function {
        name: "pred_tuple".to_string(),
        blocks: vec![
            Block {
                label: "initially".to_string(),
                body: vec![
                    call_r(1),
                    Stmt::Call {
                        name: "shift_left_put_-".to_string(),
                    },
                    assign!(lv!(@), rv!(const S::X)),
                    Stmt::Rt,
                    // if @ == 'l' { @ := '-' }
                    Stmt::Break {
                        cond: cond!(rv!(@), rv!(const S::X)),
                    },
                    assign!(lv!(@), rv!(const S::B)),
                ],
            },
            Block {
                label: "finally".to_string(),
                body: vec![call_l(1)],
            },
        ],
    }
}

// ... ? |x| - l(n) A x - ...
// ... ?  x  l  x - l(n-1) A x - l(n - 1) A x ... x - l A |x| A x - ...
pub(crate) fn expand_arg() -> Function {
    Function {
        name: "expand_arg".to_string(),
        blocks: vec![
            Block {
                label: "initially".to_string(),
                body: vec![
                    Stmt::Call {
                        name: "insert_sig".to_string(),
                    },
                    call_r(1),
                ],
            },
            Block {
                label: "check_non_zero".to_string(),
                body: vec![
                    Stmt::Rt,
                    Stmt::Rt,
                    Stmt::Jump {
                        label: "pred_if_non_zero".to_string(),
                        cond: cond!(rv!(@), rv!(const S::L)),
                    },
                    Stmt::Jump {
                        label: "pred_if_zero".to_string(),
                        cond: None,
                    },
                ],
            },
            Block {
                label: "pred_if_non_zero_and_loopback".to_string(),
                body: vec![
                    Stmt::Lt,
                    Stmt::Lt,
                    Stmt::Call {
                        name: "pred_tuple".to_string(),
                    },
                    Stmt::Call {
                        name: "copy_1".to_string(),
                    },
                    call_r(1),
                    Stmt::Jump {
                        label: "check_non_zero".to_string(),
                        cond: None,
                    },
                ],
            },
            Block {
                label: "pred_zero".to_string(),
                body: vec![
                    Stmt::Lt,
                    Stmt::Lt,
                    Stmt::Call {
                        name: "pred_tuple".to_string(),
                    },
                ],
            },
            // back to x l x
            Block {
                label: "back_to_xlx".to_string(),
                body: vec![
                    call_l(1),
                    Stmt::Rt,
                    Stmt::Break {
                        cond: cond!(rv!(@), rv!(const S::L)),
                    },
                ],
            },
            Block {
                label: "finally".to_string(),
                body: vec![call_r(1)],
            },
        ],
    }
}

// ... ? x  l  x - l(n) A x - l(n - 1) A x ... x - l A x - A |x| A x - ...
pub(crate) fn primitive_recursion(zero: String, succ: String) -> Function {
    let blocks = vec![
        Block {
            label: "call_zero".to_string(),
            body: vec![Stmt::Call { name: zero.clone() }],
            // ... ? x  l  x - l(n) A x - l(n - 1) A x ... x - l A x - A |x| - U(zero(p)) x - ...
            //      - U(zero(p)) == F(P[zero, succ](0, A))
        },
        Block {
            label: "loop".to_string(),
            body: vec![
                // x A |x| B x
                call_l(1),
                Stmt::Rt,
                Stmt::Break {
                    cond: cond!(rv!(@), rv!(const S::L)),
                },
                // non xl => |x| A x B x
                Stmt::Lt,
                Stmt::Call {
                    name: "swap_tuple".to_string(),
                },
                Stmt::Call {
                    name: "concat".to_string(),
                },
                // |x| B A x
                Stmt::Call { name: succ.clone() },
                Stmt::Continue { cond: None },
            ],
        },
        // ? x |l| x A x
        Block {
            label: "finally".to_string(),
            body: vec![
                Stmt::Rt,
                assign!(lv!(@), rv!(const S::B)),
                // x l |-| A x
                call_r(1),
                Stmt::Call {
                    name: "shift_left_put_-".to_string(),
                },
                // |l| - A x
                assign!(lv!(@), rv!(const S::X)),
                // |x| - A x
                call_r(1),
                Stmt::Call {
                    name: "shift_left_put_-".to_string(),
                },
                // |-| A x
                assign!(lv!(@), rv!(const S::X)),
                // |x| A x
            ],
        },
    ];

    Function {
        name: format!("prim_{zero}_{succ}"),
        blocks,
    }
}
