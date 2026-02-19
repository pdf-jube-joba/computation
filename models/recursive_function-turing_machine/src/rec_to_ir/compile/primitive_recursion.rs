use crate::rec_tm_ir::{Block, Function, Stmt};
use crate::rec_to_ir::S;
use crate::rec_to_ir::auxiliary::basic::{call_l, call_r};
use crate::{assign, cond, lv, rv};

// ... ? |x| - l(n) A x - ...
// ... ? |x| l x - l(n) A x - ...
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
                    call_l(2),
                ],
            },
        ],
    }
}

// ... ? |x| l x - l(n) A x - ...
// ... ? |x| l x - l(n) A x - l(n - 1) A x ... x - l A x A x - ...
pub fn expand_arg() -> Function {
    Function {
        name: "expand_arg".to_string(),
        blocks: vec![
            
        ],
    }
}
