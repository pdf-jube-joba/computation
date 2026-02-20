use crate::rec_tm_ir::{Block, Function, Stmt};
use crate::rec_to_ir::S;
use crate::rec_to_ir::auxiliary::basic::{call_l, call_r};
use crate::{assign, cond, lv, rv};

fn mu_recursion(func: String) -> Function {
    Function {
        name: format!("mu_{func}"),
        blocks: vec![
            Block {
                label: "initially".to_string(),
                body: vec![
                    call_r(1),
                    Stmt::Rt,
                    assign!(lv!(@), rv!( const S::B)),
                    Stmt::Lt,
                    assign!(lv!(@), rv!( const S::X)),
                    Stmt::Lt,
                    call_l(1),
                ],
            },
            Block {
                label: "generate_and_call".to_string(),
                body: vec![
                    // x F(p) |x| - l(n) x
                    Stmt::Call {
                        name: format!("copy_to_end_0"),
                    },
                    call_l(1),
                    Stmt::Call {
                        name: format!("copy_to_end_2"),
                    },
                    call_r(2),
                    Stmt::Call {
                        name: format!("concat"),
                    },
                    // x F(p) x - l(n) |x| - l(n) F(p) x
                    Stmt::Call { name: func },
                    // x F(p) x - l(n) |x| - l(k) x
                    Stmt::Rt,
                    Stmt::Rt,
                    Stmt::Jump {
                        label: "is_zero".to_string(),
                        cond: cond!(rv!(@), rv!(const S::X)),
                    },
                    // not zero
                    call_r(1),
                ],
            },
            Block {
                label: "is_non_zero".to_string(),
                body: vec![
                    // x F(p) x - l(n) x - l(n > 0) |x|
                    assign!(lv!(@), rv!(const S::B)),
                    Stmt::Lt,
                    Stmt::Break {
                        cond: cond!(rv!(@), rv!(const S::X)),
                    },
                    Stmt::Continue { cond: None },
                ],
            },
            Block {
                label: "next_num".to_string(),
                body: vec![
                    // x F(p) x - l(n) |x|
                    assign!(lv!(@), rv!(const S::L)),
                    Stmt::Rt,
                    assign!(lv!(@), rv!(const S::X)),
                    call_l(1),
                    Stmt::Jump {
                        label: "generate_and_call".to_string(),
                        cond: None,
                    },
                ],
            },
            Block {
                label: "is_zero".to_string(),
                body: vec![
                    // x F(p) x - l(n) x - |x|
                    assign!(lv!(@), rv!(const S::B)),
                    call_l(3),
                    Stmt::Call {
                        name: "swap_tuple".to_string(),
                    },
                    // |x| - l(n) x F(p) x
                    call_r(1),
                    Stmt::Call {
                        name: "delete".to_string(),
                    },
                ],
            },
        ],
    }
}
