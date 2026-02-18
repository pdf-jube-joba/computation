use crate::{
    rec_tm_ir::{Function, Stmt},
    rec_to_ir::S,
};

pub fn projection(n: usize, i: usize) -> Function {
    let mut body = vec![
        Stmt::Call {
            name: "rotate".to_string(),
            args: vec![]
        };
        i
    ];

    for _ in 0..n - i - 1 {
        body.extend(vec![
            Stmt::Loop {
                label: "until_x".to_string(),
                body: vec![
                    Stmt::Rt,
                    Stmt::StorConst(S::B.into()),
                    Stmt::IfBreakHead {
                        value: S::X.into(),
                        label: "until_x".to_string(),
                    },
                ],
            },
            Stmt::StorConst(S::B.into()),
        ]);
    }

    body.push(Stmt::Call {
        name: "move_right_till_x_2".to_string(),
        args: vec![],
    });

    Function {
        name: format!("proj_{}_{}", n, i),
        params: vec![],
        body,
    }
}
