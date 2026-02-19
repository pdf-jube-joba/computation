use crate::rec_tm_ir::{Block, Condition, Function, LValue, RValue, Stmt};
use crate::rec_to_ir::S;
use crate::rec_to_ir::auxiliary::basic::{call_l, call_r};
use crate::{assign, cond, lv, rv};

// copy given tuples
// input:   ... ? |x| A x B[1] x ... x B[n] x - ...
// output:  ... ? |x| A x B[1] x ... x B[n] x A x - ...
// where A is "tuple" represented by the list of {'-', 'l'} not containing 'x'
pub(crate) fn copy_to_end(n: usize) -> Function {
    Function {
        name: format!("copy_to_end_{n}"),
        // idea
        // - mark "copy-from-here" by 'x'
        // - mark "put-here" by 'x'
        // - hold "what to put" in variable "put"
        // input: ... ? [x] ?1 ?2 ?3 x B[1] x ... x B[n] x - ...
        blocks: vec![
            Block {
                label: "initially".to_string(),
                body: vec![
                    // step 1: move to right of last 'x'
                    call_r(n + 1),
                    Stmt::Rt,
                    //      ... ? x ?1 ?2 ?3 x B[1] x ... x B[n] x [-] ...
                    // @head := 'x' (mark "put-here")
                    assign!(lv!(@), rv!(const S::X)),
                    //      ... ? x ?1 ?2 ?3 x B[1] x ... x B[n] x [x] - ...
                    // and move to first x,
                    call_l(n + 2),
                    //      ... ? [x] ?1 ?2 ?3 x B[1] x ... x B[n] x x - ...
                ],
            },
            // loop label "main"
            Block {
                label: "main".to_string(),
                body: vec![
                    // step 2: move right
                    Stmt::Rt,
                    //      ... ?  x [?1] ?2 ?3 x B[1] x ... x B[n] x x - ...
                    // and if head == 'x' break loop (goto step8)
                    Stmt::Break {
                        cond: cond!(rv!(@), rv!(const S::X)),
                    },
                    // else cases

                    // step 3: put = @head, @head = 'x' (mark "copy-from-here")
                    assign!(lv!("put"), rv!(@)),
                    assign!(lv!(@), rv!(const S::X)),
                    //      ... ?  x [ x] ?2 ?3 x B[1] x ... x B[n] x x - ..., and put = ?1
                    // step 4: move right_x
                    call_r(n + 2),
                    //      ... ?  x  x  ?2 ?3 x B[1] x ... x B[n] x [x] - ..., and put = ?1
                    // and @head = put (write "what to put" to "put-here")
                    assign!(lv!(@), rv!("put")),
                    //      ... ?  x  x  ?2 ?3x B[1] x ... x B[n] x [?1] - ..., and put = ?1
                    // step 5: move right and @head = 'x'(mark "put-here")
                    Stmt::Rt,
                    assign!(lv!(@), rv!(const S::X)),
                    //      ... ?  x  x  ?2 ?3 x B[1] x ... x B[n] x ?1 [x] - ..., and put = ?1
                    // step 6: move to mark "put-here"
                    call_l(n + 2),
                    //      ... ?  x [x] ?2 ?3 x B[1] x ... x B[n] x ?1 x - ..., and put = ?1
                    // and write put (write "what to put" to "copy-from-here", "recover")
                    assign!(lv!(@), rv!("put")),
                    //      ... ?  x [?1] ?2 ?3 x B[1] x ... x B[n] x ?1 x - ..., and put = ?1
                    Stmt::Continue { cond: None },
                ],
            },
            // step 8: move left_x, and return
            //      ... ? [x] ?1 ?2   ?3   x  ?1  ?2   ?3  x B[1] x ... x B[n]  x - ..., and put = ?3
            Block {
                label: "finally".to_string(),
                body: vec![call_l(1)],
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
        let mut v = vec![];
        for i in 0..n {
            v.push(Block {
                label: format!("copy_head_{i}"),
                body: vec![
                    Stmt::Call {
                        name: "copy_to_end_0".to_string(),
                    },
                    call_r(1),
                ],
            })
        }
        for i in 0..n {
            v.push(Block {
                label: format!("copy_tail_{i}"),
                body: vec![call_l(1)],
            });
        }
        v
    };

    Function {
        name: format!("copy_{n}"),
        blocks: closure(n),
    }
}

pub(crate) fn copy(n: usize) -> Stmt {
    Stmt::Call {
        name: format!("copy_{n}"),
    }
}
