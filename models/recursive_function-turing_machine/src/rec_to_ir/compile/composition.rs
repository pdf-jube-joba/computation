use crate::rec_to_ir::auxiliary::basic::{call_l, call_r};
use crate::{assign, cond, lv, rv};
use crate::{
    rec_tm_ir::{Block, Function, Stmt},
    rec_to_ir::auxiliary::copy::copy,
};

// ... ? x A x - ...
// ... ? x outer(inner[0] A, ..., inner[n] A,) x - ...
pub fn composition(inner_functions: Vec<String>, outer_function: String) -> Function {
    let n = inner_functions.len();
    let name = format!("comp_{}_{}", outer_function, inner_functions.join("_"));
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

    for inner_function in inner_functions {
        blocks.push(Block {
            label: format!("call_{inner_function}"),
            body: vec![
                call_r(n - 1),
                Stmt::Call {
                    name: inner_function,
                },
                call_l(n - 1),
                Stmt::Call {
                    name: "rotate".to_string(),
                },
            ],
        });
    }

    blocks.push(Block {
        label: "outer".to_string(),
        body: vec![Stmt::Call {
            name: outer_function,
        }],
    });

    Function { name, blocks }
}
