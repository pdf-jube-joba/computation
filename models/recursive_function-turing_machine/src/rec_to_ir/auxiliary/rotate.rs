use crate::rec_tm_ir::{Block, Function, Stmt, register_function};
use crate::rec_to_ir::S;
use crate::rec_to_ir::auxiliary::basic::{call_l, call_r};
use crate::{assign, cond, lv, rv};

// input:   ... ? |x| A x B x - ...
// output:  ... ? |x| B x A x - ...
// where A, B are "tuple" represented by the list of {'-', 'l'} not containing 'x'
pub(crate) fn swap_tuple() -> Function {
    Function {
        name: "swap_tuple".to_string(),
        blocks: vec![
            Block {
                label: "initially".to_string(),
                body: vec![
                    // put := 'x'
                    assign!(lv!("put"), rv!(const S::X)),
                    // moves to last of 'x'
                    call_r(2),
                    // ... ? x A x B |x| - ...
                ],
            },
            Block {
                label: "loop_1".to_string(),
                body: vec![
                    // swap "put" and @head
                    assign!(lv!("tmp"), rv!(@)),
                    assign!(lv!(@), rv!("put")),
                    assign!(lv!("put"), rv!("tmp")),
                    // left
                    Stmt::Lt,
                    Stmt::Break {
                        cond: cond!(rv!("put"), rv!(const S::X)),
                    },
                    Stmt::Continue { cond: None },
                ],
            },
            Block {
                label: "loop_2".to_string(),
                body: vec![
                    // swap "put" and @head
                    assign!(lv!("tmp"), rv!(@)),
                    assign!(lv!(@), rv!("put")),
                    assign!(lv!("put"), rv!("tmp")),
                    // left
                    Stmt::Lt,
                    Stmt::Break {
                        cond: cond!(rv!("put"), rv!(const S::X)),
                    },
                    Stmt::Continue { cond: None },
                ],
            },
            Block {
                label: "loop_3".to_string(),
                body: vec![
                    // swap "put" and @head
                    assign!(lv!("tmp"), rv!(@)),
                    assign!(lv!(@), rv!("put")),
                    assign!(lv!("put"), rv!("tmp")),
                    // left
                    Stmt::Lt,
                    Stmt::Break {
                        cond: cond!(rv!("put"), rv!(const S::X)),
                    },
                    Stmt::Continue { cond: None },
                ],
            },
            Block {
                label: "left_of_left".to_string(),
                body: vec![
                    // return to "initial position"
                    Stmt::Rt,
                    Stmt::Return {
                        cond: cond!(rv!(@), rv!(const S::X)),
                    },
                    // swap "put" and @head
                    assign!(lv!("tmp"), rv!(@)),
                    assign!(lv!(@), rv!("put")),
                    assign!(lv!("put"), rv!("tmp")),
                    // return to left of last 'x'
                    call_r(3),
                    Stmt::Lt,
                    // return
                    Stmt::Jump {
                        label: "loop_1".to_string(),
                        cond: None,
                    },
                ],
            },
        ],
    }
}

// input:   ... ? |x| A[0] x A[1] x ... A[n - 1] x - ...
// output:  ... ? |x| A[1] x A[2] x ... A[n - 1] x A[0] x - ...
// where A[i] is "tuple" represented by the list of {'-', 'l'} not containing 'x'
pub(crate) fn rotate(n: usize) -> Function {
    if n == 0 || n == 1 {
        return Function {
            name: format!("rotate_{n}"),
            blocks: vec![],
        };
    }

    let swap_tuple_func = register_function(swap_tuple()).unwrap();
    let mut blocks = vec![];

    for i in 0..n - 1 {
        blocks.push(Block {
            label: format!("rot_swap_{i}"),
            body: vec![
                Stmt::Call {
                    func: swap_tuple_func.clone(),
                },
                call_r(1),
            ],
        })
    }

    blocks.push(Block {
        label: "return_to_init".to_string(),
        body: vec![call_l(n - 1)],
    });

    Function {
        name: format!("rotate_{n}"),
        blocks,
    }
}
