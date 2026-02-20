use crate::rec_tm_ir::{Block, Function, Stmt};
use crate::rec_to_ir::S;
use crate::rec_to_ir::auxiliary::basic::{call_l, call_r};
use crate::{assign, cond, lv, rv};

// ... ? |x| - l(n) A x - ...
// ... ?  x l |x| - l(n) A x - ...
pub fn insert_sig() -> Function {
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

// ... ? x l |x| - l(n) A x - ...
// ... ? x l  x - l(n) A x - l(n - 1) A x ... x - l A |x| A x - ...
pub fn expand_arg() -> Function {
    Function {
        name: "expand_arg".to_string(),
        blocks: vec![
            // ? x l x ... l (n + 1) A |x| - l(n) A x - ...
            Block {
                label: "copy_move".to_string(),
                body: vec![
                    Stmt::Call {
                        name: "copy_1".to_string(),
                    },
                    call_r(2),
                    // ? x l x ... l (n+1) A  x  - l(n) A x - l(n) A |x| - ...
                    assign!(lv!(@), rv!(const S::B)),
                    assign!(lv!("put"), rv!(const S::X)),
                ],
            },
            // ? x l x ... l (n+1) A  x  - l(n) A x - l(n) A |-| -, put = 'x'
            Block {
                label: "shift_left".to_string(),
                body: vec![
                    Stmt::Lt,
                    // swap
                    Stmt::Break {
                        cond: cond!(rv!(@), rv!(const S::X)),
                    },
                    assign!(lv!("tmp"), rv!(@)),
                    assign!(lv!(@), rv!("put")),
                    assign!(lv!("put"), rv!("tmp")),
                    Stmt::Continue { cond: None },
                ],
            },
            // ? x l x ... l(n+1) A  x  - l(n) A |x| l(n) A x - ..., put = '-'
            Block {
                label: "check".to_string(),
                body: vec![
                    Stmt::Rt,
                    Stmt::Jump {
                        label: "l_before".to_string(),
                        cond: cond!(rv!(@), rv!(const S::L)),
                    },
                    Stmt::Jump {
                        label: "finally".to_string(),
                        cond: None,
                    },
                ],
            },
            // ? x l x ... l(n+1) A  x  - l(n) A x |l| l(n - 1) A x - ..., put = '-'
            Block {
                label: "l_before".to_string(),
                body: vec![
                    assign!(lv!(@), rv!(const S::B)),
                    call_r(1),
                    assign!(lv!(@), rv!(const S::B)),
                    assign!(lv!("put"), rv!(const S::X)),
                    // ? x l x ... l (n+1) A  x  - l(n) A x - l(n - 1) A |-| -, put = 'x'
                    Stmt::Jump {
                        label: "shift_left".to_string(),
                        cond: None,
                    },
                ],
            },
            // ? x l x ... l(n+1) A  x - A x |A| x - ..., put = '-'
            Block {
                label: "finally".to_string(),
                body: vec![Stmt::Lt],
            },
        ],
    }
}

// ... ? x  l  x - l(n) A x - l(n - 1) A x ... x - l A x - A |x| A x - ...
fn call_iter_function(zero: String, succ: String) -> Function {
    let mut blocks = vec![
        Block {
            label: "call_zero".to_string(),
            body: vec![Stmt::Call { name: zero.clone() }],
            // ... ? x  l  x - l(n) A x - l(n - 1) A x ... x - l A x - A |x| - U(zero(p)) x - ...
            //      - U(zero(p)) == F(P[zero, succ](0, A))
        },
        Block {
            label: "check_left_xlx".to_string(),
            body: vec![
                Stmt::Lt,
                Stmt::Break {
                    cond: cond!(rv!(@), rv!(const S::B)),
                },
                Stmt::Break {
                    cond: cond!(rv!(@), rv!(const S::X)),
                },
                Stmt::Lt,
                Stmt::Break {
                    cond: cond!(rv!(@), rv!(const S::B)),
                },
                Stmt::Break {
                    cond: cond!(rv!(@), rv!(const S::L)),
                },
                // |x| l x A x
                Stmt::Jump {
                    label: "finally".to_string(),
                    cond: None,
                },
            ],
        },
        // ... ? x  l  x - l(n) A x - l(n - 1) A x ... x - l(i) A |x| F(P[zero, succ](i, A)) x - ...
        Block {
            label: "format".to_string(),
            body: vec![
                call_l(1),
                Stmt::Call {
                    name: "swap_tuple".to_string(),
                },
                Stmt::Call {
                    name: "concat".to_string(),
                },
                // ... ? x  l  x - l(n)  A x ... |x| F(P[zero, succ](i, A)) - l(i) A x - ...
                //  F(P[zero, succ](i, A)) - l(i) A == F(P[zero, succ](i, A)), i A)
                Stmt::Call { name: succ.clone() },
                // ... ? x  l  x - l(n)  A x ... |x| F(succ(P[zero, succ](i, A)), i, A)) x - ...
                //  succ(P[zero, succ](i, A)), i, A) == P[zero, succ](i + 1, A)
            ],
        },
        Block {
            label: "not_xlx".to_string(),
            body: vec![],
        },
        // ? |x| l x A x
        Block {
            label: "finally".to_string(),
            body: vec![],
        },
    ];

    Function {
        name: format!("prim_{zero}_{succ}"),
        blocks,
    }
}
