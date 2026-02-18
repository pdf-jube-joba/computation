use crate::rec_tm_ir::{Function, Stmt};
use crate::rec_to_ir::S;

fn call_move_right_till_x(n: usize) -> Stmt {
    Stmt::Call {
        name: format!("move_right_till_x_{n}"),
        args: vec![],
    }
}

fn call_move_left_till_x(n: usize) -> Stmt {
    Stmt::Call {
        name: format!("move_left_till_x_{n}"),
        args: vec![],
    }
}

// copy given tuples
// input:   ... ? |x| A x B[1] x ... x B[n] x - ...
// output:  ... ? |x| A x B[1] x ... x B[n] x A - ...
// where A is "tuple" represented by the list of {'-', 'l'} not containing 'x'
pub(crate) fn copy_to_end(n: usize) -> Function {
    Function {
        name: format!("copy_to_end_{n}"),
        params: vec![],
        // idea
        // - mark "copy-from-here" by 'x'
        // - mark "put-here" by 'x'
        // - hold "what to put" in variable "put"
        // input: ... ? [x] ?1 ?2 ?3 x B[1] x ... x B[n] x - ...
        body: vec![
            // step 1: move right_x, move right
            call_move_right_till_x(n + 1),
            // call_move_right_till_x(),
            Stmt::Rt,
            //      ... ? x ?1 ?2 ?3 x B[1] x ... x B[n] x [-] ...
            // write 'x'(mark "put-here")
            Stmt::StorConst(S::X.into()),
            //      ... ? x ?1 ?2 ?3 x B[1] x ... x B[n] x [x] - ...
            // and move left_x twice,
            call_move_left_till_x(n + 2),
            // call_move_left_till_x(),
            // call_move_left_till_x(),
            //      ... ? [x] ?1 ?2 ?3 x B[1] x ... x B[n] x x - ...
            // loop label "until_x"
            Stmt::Loop {
                label: "until_x".to_string(),
                body: vec![
                    // step 2: move right
                    Stmt::Rt,
                    //      ... ?  x [?1] ?2 ?3 x B[1] x ... x B[n] x x - ...
                    // and if head == 'x' break loop (goto step8)
                    Stmt::IfBreakHead {
                        value: S::X.into(),
                        label: "until_x".to_string(), // step8
                    },
                    // else cases

                    // step 3: put = @head, write 'x'(mark "copy-from-here")
                    Stmt::Read("put".to_string()),
                    Stmt::StorConst(S::X.into()),
                    //      ... ?  x [ x] ?2 ?3 x B[1] x ... x B[n] x x - ..., and put = ?1
                    // step 4: move right_x twice
                    call_move_right_till_x(n + 2),
                    // call_move_right_till_x(),
                    // call_move_right_till_x(),
                    //      ... ?  x  x  ?2 ?3 x B[1] x ... x B[n] x [x] - ..., and put = ?1
                    // and write put (write "what to put" to "put-here")
                    Stmt::Stor("put".to_string()),
                    //      ... ?  x  x  ?2 ?3x B[1] x ... x B[n] x [?1] - ..., and put = ?1
                    // step 5: move right and write 'x'(mark "put-here")
                    Stmt::Rt,
                    Stmt::StorConst(S::X.into()),
                    //      ... ?  x  x  ?2 ?3 x B[1] x ... x B[n] x ?1 [x] - ..., and put = ?1
                    // step 6: move left_x twice -> current: "copy-from-here"
                    call_move_left_till_x(n + 2),
                    // call_move_left_till_x(),
                    // call_move_left_till_x(),
                    //      ... ?  x [x] ?2 ?3 x B[1] x ... x B[n] x ?1 x - ..., and put = ?1
                    // and write put (write "what to put" to "copy-from-here", "recover")
                    Stmt::Stor("put".to_string()),
                    //      ... ?  x [?1] ?2 ?3 x B[1] x ... x B[n] x ?1 x - ..., and put = ?1
                    // loop back to step2
                ],
            },
            // step 8: move left_x, and return
            //      ... ? [x] ?1 ?2   ?3   x  ?1  ?2   ?3  x B[1] x ... x B[n]  x - ..., and put = ?3
            Stmt::Call {
                name: "move_left_till_x_1".to_string(),
                args: vec![],
            },
        ],
    }
}

// ... ? x A x - ...
// ... ? x A x A x ... x A x - ... A * n times
pub(crate) fn copy_n_times(n: usize) -> Function {
    // here we "generate" the body by repeating copy and move_right_till_x, and move_lefts at the end
    // compile time recursive calls are fine since n is known at compile time
    // intuition:
    //  copy_n_times(0) = vec![]
    //  copy_n_times(n + 1) = copy() + move_right_till_x() + copy_n_times(n) + move_left_till_x() for n > 0

    let closure = |n: usize| {
        if n == 0 {
            vec![]
        } else {
            let mut body = copy_n_times(n - 1).body;
            body.insert(0, call_move_right_till_x(1));
            body.insert(
                0,
                Stmt::Call {
                    name: "copy_to_end_0".to_string(),
                    args: vec![],
                },
            );
            body.push(call_move_left_till_x(1));
            body
        }
    };

    Function {
        name: format!("copy_{n}"),
        params: vec![],
        body: closure(n),
    }
}

pub(crate) fn copy_one() -> Stmt {
    Stmt::Call {
        name: "copy_0".to_string(),
        args: vec![],
    }
}
