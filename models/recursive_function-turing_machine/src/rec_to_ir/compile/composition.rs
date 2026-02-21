use crate::rec_tm_ir::register_function;
use crate::rec_to_ir::auxiliary::basic::{call_l, call_r};
use crate::{assign, cond, lv, rv};
use crate::{
    rec_tm_ir::{Block, Function, Stmt, get_function},
    rec_to_ir::auxiliary::copy::copy,
};

// ... ? x A x - ...
// ... ? x outer(inner[0] A, ..., inner[n] A,) x - ...
pub fn composition(inner_functions: Vec<Function>, outer_function: Function) -> Function {
    let n = inner_functions.len();
    let name = format!(
        "comp_{}_{}",
        outer_function.name,
        inner_functions
            .iter()
            .map(|fnc| fnc.name.to_string())
            .collect::<Vec<_>>()
            .join("_")
    );
    let mut blocks = vec![
        Block {
            label: "initially".to_string(),
            body: vec![
                // copy inner_functions.len() times
                copy(n),
            ],
        }, // ... ? x A x A x ... x A x - ... A * n times
           // ... ? x A x ... |x| A x - ...
    ];

    let rotate_func = get_function("rotate").unwrap();
    for inner_function in inner_functions {
        let inner_func = register_function(inner_function).unwrap();
        blocks.push(Block {
            label: format!("call_{}", inner_func.name),
            body: vec![
                call_r(n - 1),
                Stmt::Call { func: inner_func },
                call_l(n - 1),
                Stmt::Call {
                    func: rotate_func.clone(),
                },
            ],
        });
    }

    let outer_func = register_function(outer_function).unwrap();
    blocks.push(Block {
        label: "outer".to_string(),
        body: vec![Stmt::Call { func: outer_func }],
    });

    Function { name, blocks }
}
