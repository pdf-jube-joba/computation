use recursive_function::machine::RecursiveFunctions;
use turing_machine::manipulation::builder::TuringMachineBuilder;

pub mod num_tape {
    use crate::symbols::S;
    use turing_machine::machine::{Sign, Tape};
    use utils::number::*;

    fn num_sings(num: Number) -> Vec<Sign> {
        (0..num.as_usize().unwrap()).map(|_| S::L.into()).collect()
    }

    pub fn write(tuple: Vec<Number>) -> Tape {
        let mut signs: Vec<Sign> = vec![];
        signs.push(S::X.into());

        for num in tuple {
            signs.push(Sign::blank());
            signs.extend_from_slice(&num_sings(num));
            signs.push(Sign::blank());
        }

        Tape::from_vec(signs, 0).unwrap()
    }

    pub fn write_usize(tuple: Vec<usize>) -> Tape {
        let number_tuple: Vec<Number> = tuple.into_iter().map(|x| x.into()).collect();
        write(number_tuple)
    }

    fn read_one(signs: Vec<Sign>) -> Option<Vec<Number>> {
        let v = signs
            .split(|char| *char == Sign::blank())
            .map(|vec| vec.len().into())
            .skip(1);
        Some(v.collect::<Vec<_>>())
    }

    pub fn read_right_one(tape: &Tape) -> Option<Vec<Number>> {
        let (v, p) = tape.into_vec();
        if v[p] != S::X.into() {
            return None;
        }

        let iter = v.into_iter().skip(p);
        read_one(iter.collect())
    }

    pub fn read_right_one_usize(tape: &Tape) -> Option<Vec<usize>> {
        read_right_one(tape).map(|vec| vec.into_iter().map(|x| x.as_usize().unwrap()).collect())
    }
}

pub fn zero_builder() -> TuringMachineBuilder {
    let mut builder = TuringMachineBuilder::new("zero_builder").unwrap();
    builder
        .from_source(include_str!("zero_builder.txt"))
        .unwrap();
    builder
}

pub fn succ_builder() -> TuringMachineBuilder {
    let mut builder = TuringMachineBuilder::new("succ_adder").unwrap();
    builder
        .from_source(include_str!("succ_builder.txt"))
        .unwrap();
    builder
}

pub mod composition;
pub mod mu_recursion;
pub mod primitive_recursion;
pub mod projection;

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

#[cfg(test)]
mod tests;
