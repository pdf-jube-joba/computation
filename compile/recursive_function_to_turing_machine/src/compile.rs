use super::builder::*;
use recursive_function::machine::RecursiveFunctions;
use turing_machine::{machine::*, manipulation::builder::TuringMachineBuilder};

pub mod num_tape {
    use recursive_function::machine::{Number, NumberTuple};
    use turing_machine::machine::{Sign, TapeAsVec};

    fn partition() -> Sign {
        Sign::try_from("-").unwrap()
    }

    fn one() -> Sign {
        Sign::try_from("1").unwrap()
    }

    fn num_sings(num: Number) -> Vec<Sign> {
        (0..num.into()).map(|_| one()).collect()
    }

    pub fn write(tuple: NumberTuple) -> TapeAsVec {
        let vec: Vec<Number> = tuple.into();
        let mut signs: Vec<Sign> = vec
            .into_iter()
            .flat_map(|num: Number| {
                let mut vec = vec![Sign::blank()];
                vec.extend_from_slice(&num_sings(num));
                vec
            })
            .collect();
        signs.extend_from_slice(&vec![partition()]);
        TapeAsVec {
            left: vec![],
            head: partition(),
            right: signs,
        }
    }

    fn read_one(signs: Vec<Sign>) -> Result<NumberTuple, ()> {
        let v = signs
            .split(|char| *char == Sign::blank())
            .map(|vec| vec.len())
            .skip(1);
        Ok(v.collect::<Vec<_>>().into())
    }

    pub fn read_right_one(tape: TapeAsVec) -> Result<NumberTuple, ()> {
        if tape.head != partition() {
            return Err(());
        }
        let iter = tape
            .right
            .iter()
            .take_while(|sign| **sign == Sign::blank() || **sign == one())
            .cloned();
        read_one(iter.collect())
    }
}

pub fn compile(recursive_function: &RecursiveFunctions) -> TuringMachineBuilder {
    match recursive_function {
        RecursiveFunctions::ZeroConstant => zero_builder(),
        RecursiveFunctions::Successor => succ_builder(),
        RecursiveFunctions::Projection(proj) => {
            projection::projection(proj.parameter_length(), proj.projection_num())
        }
        RecursiveFunctions::Composition(composition) => {
            let recursive_function::machine::Composition {
                parameter_length: _,
                outer_func,
                inner_func,
            } = composition;
            let outer_builder = compile(outer_func.as_ref());
            let inner_builders: Vec<TuringMachineBuilder> = inner_func
                .to_owned()
                .into_iter()
                .map(|func| compile(&func))
                .collect();
            composition::composition(inner_builders, outer_builder)
        }
        RecursiveFunctions::PrimitiveRecursion(prim) => {
            let recursive_function::machine::PrimitiveRecursion {
                zero_func,
                succ_func,
            } = prim;
            primitive_recursion::primitive_recursion(
                compile(zero_func.as_ref()),
                compile(succ_func.as_ref()),
            )
        }
        RecursiveFunctions::MuOperator(muop) => {
            let recursive_function::machine::MuOperator { mu_func } = muop;
            mu_recursion::mu_recursion(compile(mu_func.as_ref()))
        }
    }
}

#[cfg(test)]
mod tests;
