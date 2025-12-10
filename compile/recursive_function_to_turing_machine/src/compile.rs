use recursive_function::machine::RecursiveFunctions;
use turing_machine::manipulation::builder::TuringMachineBuilder;

pub mod num_tape {
    use turing_machine::machine::{Sign, Tape};
    use utils::number::*;

    fn partition() -> Sign {
        "-".parse().unwrap()
    }

    fn one() -> Sign {
        "1".parse().unwrap()
    }

    fn num_sings(num: Number) -> Vec<Sign> {
        (0..num.into()).map(|_| one()).collect()
    }

    pub fn write(tuple: Vec<Number>) -> Tape {
        let mut signs: Vec<Sign> = tuple
            .into_iter()
            .flat_map(|num: Number| {
                let mut vec = vec![Sign::blank()];
                vec.extend_from_slice(&num_sings(num));
                vec
            })
            .collect();
        signs.extend_from_slice(&[partition()]);
        Tape {
            left: vec![],
            head: partition(),
            right: signs,
        }
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
        if tape.head != partition() {
            return None;
        }
        eprintln!("hello");
        let iter = tape
            .right
            .iter()
            .take_while(|sign| **sign == Sign::blank() || **sign == one())
            .cloned();
        read_one(iter.collect())
    }

    pub fn read_right_one_usize(tape: &Tape) -> Option<Vec<usize>> {
        read_right_one(tape).map(|vec| vec.into_iter().map(|x| x.0).collect())
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
            let inner_builders: Vec<TuringMachineBuilder> = inner_funcs
                .iter()
                .cloned()
                .map(|func| compile(&func))
                .collect();
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
