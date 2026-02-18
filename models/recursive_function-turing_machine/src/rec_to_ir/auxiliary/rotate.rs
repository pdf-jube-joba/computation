use crate::rec_tm_ir::{Function, Stmt};
use crate::rec_to_ir::S;

fn call_move_right_till_x() -> Stmt {
    Stmt::Call {
        name: "move_right_till_x_1".to_string(),
        args: vec![],
    }
}

fn call_move_left_till_x() -> Stmt {
    Stmt::Call {
        name: "move_left_till_x_1".to_string(),
        args: vec![],
    }
}

// input:   ... ? |x| A x B x - ...
// output:  ... ? |x| B x A x - ...
// where A, B are "tuple" represented by the list of {'-', 'l'} not containing 'x'
pub(crate) fn swap_tuple() -> Function {
    Function {
        name: "swap_tuple".to_string(),
        params: vec![],
        body: vec![
            // put := 'x'
            Stmt::ConstAssign("put".to_string(), S::X.into()),
            // moves to the right of B
            call_move_right_till_x(),
            call_move_right_till_x(),
            // ... ? x A x B |x| - ...
            // first loop
            Stmt::Loop {
                label: "first_loop".to_string(),
                body: vec![Stmt::Loop {
                    label: "insert_swap_1".to_string(),
                    body: vec![
                        Stmt::Lt,
                        // swap head and "put"
                        Stmt::Read("tmp".to_string()),
                        Stmt::Stor("put".to_string()),
                        Stmt::Assign("put".to_string(), "tmp".to_string()),
                        // if "put" == 'x' goto insert_swap_2
                        Stmt::IfBreak {
                            var: "put".to_string(),
                            value: S::X.into(),
                            label: "insert_swap_1".to_string(),
                        },
                    ],
                }],
            },
            Stmt::Loop {
                label: "main_loop".to_string(),
                body: vec![
                    Stmt::Loop {
                        label: "insert_swap_1".to_string(),
                        body: vec![
                            Stmt::Lt,
                            // swap head and "put"
                            Stmt::Read("tmp".to_string()),
                            Stmt::Stor("put".to_string()),
                            Stmt::Assign("put".to_string(), "tmp".to_string()),
                            // if "put" == 'x' goto insert_swap_2
                            Stmt::IfBreak {
                                var: "put".to_string(),
                                value: S::X.into(),
                                label: "insert_swap_1".to_string(),
                            },
                        ],
                    },
                    Stmt::Lt,
                    Stmt::Loop {
                        label: "insert_swap_2".to_string(),
                        body: vec![
                            Stmt::IfBreakHead {
                                value: S::X.into(),
                                label: "insert_swap_2".to_string(),
                            },
                            // swap head and "put"
                            Stmt::Read("tmp".to_string()),
                            Stmt::Stor("put".to_string()),
                            Stmt::Assign("put".to_string(), "tmp".to_string()),
                            // if "put" == 'x' goto insert_swap_2
                            Stmt::Lt,
                        ],
                    },
                    Stmt::IfBreak {
                        var: "put".to_string(),
                        value: S::X.into(),
                        label: "main_loop".to_string(),
                    },
                    call_move_right_till_x(),
                    call_move_right_till_x(),
                    call_move_right_till_x(),
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
            params: vec![],
            body: vec![],
        };
    }

    let mut body = vec![];

    for _ in 0..n - 1 {
        body.push(Stmt::Call {
            name: "swap_tuple".to_string(),
            args: vec![],
        });
        body.push(call_move_right_till_x());
    }

    for _ in 0..n - 1 {
        body.push(call_move_left_till_x());
    }

    Function {
        name: format!("rotate_{n}"),
        params: vec![],
        body,
    }
}
