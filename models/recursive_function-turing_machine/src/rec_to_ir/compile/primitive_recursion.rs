use crate::{
    rec_tm_ir::{Function, Stmt},
    rec_to_ir::{
        S,
        auxiliary::{
            basic::{call_l, call_r},
            copy::{self, copy_one},
        },
    },
};

// ... ? |x| - l(n) A x - ...
// ... ? |x| l x - l(n) A x - ...
pub fn insert_sig() -> Function {
    Function {
        name: "insert_sig".to_string(),
        params: vec![],
        body: vec![
            Stmt::ConstAssign("tmp1".to_string(), S::L.into()),
            Stmt::ConstAssign("tmp2".to_string(), S::X.into()),
            Stmt::Loop {
                label: "until_x".to_string(),
                body: vec![],
            },
            Stmt::Stor("tmp1".to_string()),
            Stmt::Rt,
            Stmt::Stor("tmp2".to_string()),
            Stmt::Rt,
            Stmt::StorConst(S::X.into()),
            call_l(2),
        ],
    }
}

// ... ? |x| l x - l(n) A x - ...
// ... ? |x| l x - l(n) A x - l(n - 1) A x ... x - l A x A x - ...
pub fn expand_arg() -> Function {
    let mut body = vec![
        // ... ? |x| l x - l(n) A x - ...
        call_r(1),
        copy_one(),
        call_r(1),
        // ... ? x l x - l(n) A |x| - l(n) A x - ...

    ];

    Function {
        name: "expand_arg".to_string(),
        params: vec![],
        body,
    }
}
