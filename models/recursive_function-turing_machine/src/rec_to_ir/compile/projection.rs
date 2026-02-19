use crate::rec_tm_ir::{Block, Function, Stmt};
use crate::rec_to_ir::S;
use crate::rec_to_ir::auxiliary::basic::{call_l, call_r};
use crate::{assign, cond, lv, rv};

// aux function
// ... ? |x| - l(n_0) - l(n_1) ... - l(n_{n - 1}) x - ...
// ... ? |x| - -(n_0) - ... - l(n_i) |x| - ...
pub fn aux_projection_init(n: usize, i: usize) -> Function {
    let mut blocks = vec![Block {
        label: "initial".to_string(),
        body: vec![Stmt::Rt],
    }];

    // 1. move to i-th '-' and flatnning

    for k in 0..i {
        blocks.push(Block {
            label: format!("flat_{k}"),
            body: vec![
                // assert current is '-' of k-th number
                assign!(lv!(@), rv!(const S::B)),
                Stmt::Rt,
                Stmt::Break {
                    cond: cond!(rv!(@), rv!(const S::B)),
                },
                Stmt::Continue { cond: None },
            ],
        });
    }

    // move to last 'x'
    blocks.push(Block {
        label: "move_x".to_string(),
        body: vec![call_r(1)],
    });

    // move to right of i-th and

    for k in (i + 1..n).rev() {
        blocks.push(Block {
            label: format!("flat_{k}"),
            body: vec![
                // assert current is right of k-th number
                assign!(lv!(@), rv!(const S::B)),
                Stmt::Lt,
                Stmt::Break {
                    cond: cond!(rv!(@), rv!(const S::B)),
                },
                Stmt::Continue { cond: None },
            ],
        });
    }

    // put 'x'
    blocks.push(Block {
        label: "push_end_x".to_string(),
        body: vec![assign!(lv!(@), rv!(const S::X))],
    });

    Function {
        name: format!("aux_proj_{n}_{i}"),
        blocks,
    }
}

// ... ? |x| - l(n_0) - l(n_1) ... - l(n_{n - 1}) x - ...
// ... ? |x| - l(n_i) x - ...
pub fn projection(n: usize, i: usize) -> Function {
    let mut blocks = vec![Block {
        label: "initial".to_string(),
        body: vec![Stmt::Call {
            name: format!("aux_proj_{n}_{i}"),
        }],
    }];
    // current situation
    //  ... x -+ l* |x| - ...

    blocks.push(Block {
        label: "loop2_init".to_string(),
        body: vec![
            Stmt::Lt,
            Stmt::Break {
                cond: cond!(rv!(@), rv!(const S::B)),
            },
            Stmt::Continue { cond: None },
        ],
    });

    // ... x -* |-| l* x

    blocks.push(Block {
        label: "check_left".to_string(),
        body: vec![
            Stmt::Lt,
            Stmt::Return {
                cond: cond!(rv!(@), rv!(const S::X)),
            },
            Stmt::Rt,
        ],
    });

    blocks.push(Block {
        label: "initial_B".to_string(),
        body: vec![
            call_r(1),
            // "what-to-put"
            assign!(lv!(@), rv!(const S::B)),
            assign!(lv!("put"), rv!(const S::X)),
        ],
    });

    blocks.push(Block {
        label: "shift_to_left_until_B".to_string(),
        body: vec![
            Stmt::Lt,
            // swap
            assign!(lv!("tmp"), rv!(@)),
            assign!(lv!(@), rv!("put")),
            assign!(lv!("put"), rv!("tmp")),
            // if what we actioned is '-' ...
            Stmt::Break {
                cond: cond!(rv!("put"), rv!(const S::B)),
            },
            Stmt::Continue { cond: None },
        ],
    });

    blocks.push(Block {
        label: "finally_B".to_string(),
        body: vec![
            Stmt::Lt,
            Stmt::Jump {
                label: "check_left".to_string(),
                cond: None,
            },
        ],
    });

    Function {
        name: format!("proj_{n}_{i}"),
        blocks,
    }
}
