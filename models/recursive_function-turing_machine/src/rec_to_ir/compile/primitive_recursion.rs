use crate::rec_tm_ir::{Block, Function, Stmt, get_function, register_function};
use crate::rec_to_ir::S;
use crate::rec_to_ir::auxiliary::basic::{self, call_l, call_r};
use crate::rec_to_ir::auxiliary::{copy, rotate};
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
    let shift_left_put_blank = register_function(basic::shift_left_x(S::B)).unwrap();
    Function {
        name: "pred_tuple".to_string(),
        blocks: vec![
            Block {
                label: "initially".to_string(),
                body: vec![
                    call_r(1),
                    Stmt::Call {
                        func: shift_left_put_blank,
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
    let insert_sig_func = register_function(insert_sig()).unwrap();
    let pred_tuple_func = register_function(pred_tuple()).unwrap();
    let copy_1_func = register_function(copy::copy_n_times(1)).unwrap();
    Function {
        name: "expand_arg".to_string(),
        blocks: vec![
            Block {
                label: "initially".to_string(),
                body: vec![
                    Stmt::Call {
                        func: insert_sig_func.clone(),
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
                        func: pred_tuple_func.clone(),
                    },
                    Stmt::Call {
                        func: copy_1_func.clone(),
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
                        func: pred_tuple_func.clone(),
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
pub(crate) fn primitive_recursion(zero: Function, succ: Function) -> Function {
    let name = format!("prim_{}_{}", zero.name, succ.name);

    let zero_func = register_function(zero).unwrap();
    let succ_func = register_function(succ).unwrap();
    let swap_tuple_func = register_function(rotate::swap_tuple()).unwrap();
    let concat_func = register_function(basic::concat()).unwrap();
    let shift_left_put_blank = register_function(basic::shift_left_x(S::B)).unwrap();

    let blocks = vec![
        Block {
            label: "call_zero".to_string(),
            body: vec![Stmt::Call { func: zero_func }],
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
                    func: swap_tuple_func.clone(),
                },
                Stmt::Call {
                    func: concat_func.clone(),
                },
                // |x| B A x
                Stmt::Call {
                    func: succ_func.clone(),
                },
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
                    func: shift_left_put_blank.clone(),
                },
                // |l| - A x
                assign!(lv!(@), rv!(const S::X)),
                // |x| - A x
                call_r(1),
                Stmt::Call {
                    func: shift_left_put_blank,
                },
                // |-| A x
                assign!(lv!(@), rv!(const S::X)),
                // |x| A x
            ],
        },
    ];

    Function { name, blocks }
}
