use crate::rec_tm_ir::{Block, Function, Stmt, get_function, register_function};
use crate::rec_to_ir::S;
use crate::rec_to_ir::auxiliary::basic::{call_l, call_r};
use crate::rec_to_ir::auxiliary::{copy, rotate};
use crate::{assign, cond, lv, rv};

pub(crate) fn mu_recursion(func: Function) -> Function {
    let copy_to_end_0 = register_function(copy::copy_to_end(0)).unwrap();
    let copy_to_end_2 = register_function(copy::copy_to_end(2)).unwrap();
    let concat_func = register_function(crate::rec_to_ir::auxiliary::basic::concat()).unwrap();

    let callee_name = format!("mu_{}", &func.name);

    let callee_func = register_function(func).unwrap();
    let swap_tuple_func = register_function(rotate::swap_tuple()).unwrap();
    let delete_func = register_function(delete()).unwrap();
    Function {
        name: callee_name,
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
                        func: copy_to_end_0.clone(),
                    },
                    call_l(1),
                    Stmt::Call {
                        func: copy_to_end_2.clone(),
                    },
                    call_r(2),
                    Stmt::Call {
                        func: concat_func.clone(),
                    },
                    // x F(p) x - l(n) |x| - l(n) F(p) x
                    Stmt::Call {
                        func: callee_func.clone(),
                    },
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
                        func: swap_tuple_func,
                    },
                    // |x| - l(n) x F(p) x
                    call_r(1),
                    Stmt::Call { func: delete_func },
                ],
            },
        ],
    }
}

// ... |x| A x - ...
// ... |x| - ...
fn delete() -> Function {
    Function {
        name: "delete".to_string(),
        blocks: vec![
            Block {
                label: "loop".to_string(),
                body: vec![
                    Stmt::Rt,
                    Stmt::Break {
                        cond: cond!(rv!(@), rv!(const S::X)),
                    },
                    assign!(lv!(@), rv!(const S::B)),
                    Stmt::Continue { cond: None },
                ],
            },
            Block {
                label: "finally".to_string(),
                body: vec![assign!(lv!(@), rv!(const S::B)), call_r(1)],
            },
        ],
    }
}
