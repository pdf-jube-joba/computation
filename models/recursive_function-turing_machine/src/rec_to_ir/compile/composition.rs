use crate::rec_tm_ir::{Function, Stmt};

// ... ? x A x - ...
// ... ? x outer(inner[0] A, ..., inner[n] A,) x - ...
pub fn composition(inner_functions: Vec<String>, outer_function: String) -> Function {
    let n = inner_functions.len();
    let name = format!("comp_{}_{}", outer_function, inner_functions.join("_"));
    let mut body = vec![
        // copy inner_functions.len() times
        Stmt::Call {
            name: format!("copy_{}", n),
            args: vec![],
        },
        // ... ? x A x A x ... x A x - ... A * n times
        // ... ? x A x ... |x| A x - ...
    ];

    for inner_function in inner_functions {
        body.extend(vec![
            Stmt::Call {
                name: format!("move_right_till_x_{}", n - 1),
                args: vec![],
            },
            Stmt::Call {
                name: inner_function,
                args: vec![],
            },
            Stmt::Call {
                name: format!("move_left_till_x_{}", n - 1),
                args: vec![],
            },
            Stmt::Call {
                name: "rotate".to_string(),
                args: vec![],
            },
        ]);
    }

    body.push(Stmt::Call {
        name: outer_function,
        args: vec![],
    });

    Function {
        name,
        params: vec![],
        body,
    }
}
