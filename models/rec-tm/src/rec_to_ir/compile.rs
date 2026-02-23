use crate::rec_tm_ir::{Block, Function, Program, Stmt, reset_registry};
use crate::rec_to_ir::S;
use crate::rec_to_ir::auxiliary::basic::{call_l, call_r};
use crate::{assign, cond, lv, rv};

// 0 定数関数
// 入力: ... ? |x| x - ...
// 出力: ... ? |x| - x - ...
pub(crate) fn zero_function() -> Function {
    Function {
        name: "zero_const_function".to_string(),
        blocks: vec![Block {
            label: "entry".to_string(),
            body: vec![
                Stmt::Rt,
                assign!(lv!(@), rv!(const S::B)),
                Stmt::Rt,
                assign!(lv!(@), rv!(const S::X)),
                // returns to be the initial position
                Stmt::Lt,
                Stmt::Lt,
            ],
        }],
    }
}

// 後者関数
// 入力: ... ? |x| - l l ... l x - ... : l * n times
// 出力: ... ? |x| - l l ... l l x - ... : l * (n+1) times
pub(crate) fn succ_function() -> Function {
    Function {
        name: "succ_function".to_string(),
        blocks: vec![Block {
            label: "entry".to_string(),
            body: vec![
                call_r(1),
                assign!(lv!(@), rv!(const S::L)),
                Stmt::Rt,
                assign!(lv!(@), rv!(const S::X)),
                // returns to be the initial position ... until the first x
                call_l(1),
            ],
        }],
    }
}

pub use projection::projection;

pub mod composition;
pub mod mu_recursion;
pub mod primitive_recursion;
pub mod projection;

use recursive_function::machine::RecursiveFunctions;

pub fn compile(recursive_function: &RecursiveFunctions) -> Function {
    match recursive_function {
        RecursiveFunctions::ZeroConstant => zero_function(),
        RecursiveFunctions::Successor => succ_function(),
        RecursiveFunctions::Projection {
            parameter_length,
            projection_num,
        } => projection::projection(*parameter_length, *projection_num),
        RecursiveFunctions::Composition {
            parameter_length: _,
            outer_func,
            inner_funcs,
        } => {
            let outer_function = compile(outer_func.as_ref());
            let inner_functions: Vec<Function> = inner_funcs.iter().map(compile).collect();
            composition::composition(inner_functions, outer_function)
        }
        RecursiveFunctions::PrimitiveRecursion {
            zero_func,
            succ_func,
        } => {
            let zero_func = compile(zero_func.as_ref());
            let succ_func = compile(succ_func.as_ref());
            primitive_recursion::primitive_recursion(zero_func, succ_func)
        }
        RecursiveFunctions::MuOperator { mu_func } => {
            let mu_func = compile(mu_func.as_ref());
            mu_recursion::mu_recursion(mu_func)
        }
    }
}

pub fn compile_to_program(recursive_function: &RecursiveFunctions) -> Program {
    reset_registry();
    let main_function = compile(recursive_function);
    crate::rec_to_ir::wrap_function(main_function)
}
