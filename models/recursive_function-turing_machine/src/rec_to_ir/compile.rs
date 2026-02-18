use super::S;
use crate::rec_tm_ir::{Function, Stmt};

// 0 定数関数
// 入力: ... ? |x| x - ...
// 出力: ... ? |x| - x - ...
pub(crate) fn zero_builder() -> Function {
    Function {
        name: "zero_const_function".to_string(),
        params: vec![],
        body: vec![
            Stmt::Rt,
            Stmt::StorConst(S::B.into()),
            Stmt::Rt,
            Stmt::StorConst(S::X.into()),
            // returns to be the initial position
            Stmt::Lt,
            Stmt::Lt,
        ],
    }
}

// 後者関数
// 入力: ... ? |x| - l l ... l x - ... : l が n 個
// 出力: ... ? |x| - l l ... l l x - ... : l が n+1 個
pub(crate) fn succ_builder() -> Function {
    Function {
        name: "succ_function".to_string(),
        params: vec![],
        body: vec![
            Stmt::Call {
                name: "move_right_till_x".to_string(),
                args: vec![],
            },
            Stmt::StorConst(S::L.into()),
            Stmt::Rt,
            Stmt::StorConst(S::X.into()),
            // returns to be the initial position ... until the first x
            Stmt::Call {
                name: "move_left_till_x".to_string(),
                args: vec![],
            },
        ],
    }
}

/*
use recursive_function::machine::RecursiveFunctions;
use turing_machine::machine::TuringMachineDefinition;
use turing_machine::manipulation::builder::TuringMachineBuilder;
use utils::parse::ParseTextCodec;

pub fn zero_builder() -> TuringMachineBuilder {
    let definition: TuringMachineDefinition = include_str!("zero_builder.txt").parse_tc().unwrap();
    let mut builder =
        TuringMachineBuilder::new("zero_builder", definition.init_state().clone()).unwrap();
    builder.accepted_state = definition.accepted_state().clone();
    builder.code = definition.code().clone();
    builder
}

pub fn succ_builder() -> TuringMachineBuilder {
    let definition: TuringMachineDefinition = include_str!("succ_builder.txt").parse_tc().unwrap();
    let mut builder =
        TuringMachineBuilder::new("succ_adder", definition.init_state().clone()).unwrap();
    builder.accepted_state = definition.accepted_state().clone();
    builder.code = definition.code().clone();
    builder
}

pub mod composition;
pub mod mu_recursion;
pub mod primitive_recursion;
pub mod projection;

#[cfg(test)]
mod tests;

pub fn compile(recursive_function: &RecursiveFunctions) -> TuringMachineBuilder {
    match recursive_function {
        RecursiveFunctions::ZeroConstant => zero_builder(),
        RecursiveFunctions::Successor => succ_builder(),
        RecursiveFunctions::Projection {
            parameter_length,
            projection_num,
        } => projection::projection(*parameter_length, *projection_num),
        RecursiveFunctions::Composition {
            parameter_length: _,
            outer_func,
            inner_funcs,
        } => {
            let outer_builder = compile(outer_func.as_ref());
            let inner_builders: Vec<TuringMachineBuilder> =
                inner_funcs.iter().map(compile).collect();
            composition::composition(inner_builders, outer_builder)
        }
        RecursiveFunctions::PrimitiveRecursion {
            zero_func,
            succ_func,
        } => primitive_recursion::primitive_recursion(
            compile(zero_func.as_ref()),
            compile(succ_func.as_ref()),
        ),
        RecursiveFunctions::MuOperator { mu_func } => {
            mu_recursion::mu_recursion(compile(mu_func.as_ref()))
        }
    }
}
*/
