use std::collections::{HashMap, HashSet};
use utils::number::*;

pub mod circuit_components;
use circuit_components::*;

pub mod logic_circuit;
use logic_circuit::*;

#[derive(Debug, Clone)]
pub struct FiniteCircuitProcess {
    circuit: FiniteLogicCircuit,
    state: CircuitState,
}

impl FiniteCircuitProcess {
    pub fn from_circuit_and_input(
        circuit: FiniteLogicCircuit,
        input_state: InputState,
    ) -> Option<Self> {
        let mut state = HashMap::new();
        for (v, l, s) in circuit.iterate_as_set() {
            if let Some(b) = s {
                state.insert(v.clone(), b.clone());
            } else if l.is_inlabel() {
                let b = input_state.get_index(v);
                state.insert(v.clone(), b.clone());
            } else {
                state.insert(v.clone(), Bool::False);
            }
        }
        Some(Self {
            circuit,
            state: state.into(),
        })
    }
    pub fn from_circuit_and_state(
        circuit: FiniteLogicCircuit,
        state: CircuitState,
    ) -> Option<Self> {
        let appered_circuit = circuit.appered_vertex();
        let appered_state = state.appered();
        if appered_circuit == appered_state {
            Some(Self { circuit, state })
        } else {
            None
        }
    }
    pub fn output_of_vertex(&self, outputlabel: &VertexNumbering) -> Option<Bool> {
        self.state.get_index(outputlabel)
    }
    pub fn output(&self) -> OutputState {
        self.circuit
            .appered_vertex_with_label()
            .into_iter()
            .filter_map(|(v, l)| if l.is_outlabel() { Some(v) } else { None })
            .map(|v| (v.clone(), self.state.get_index(&v).unwrap()))
            .into()
    }
    pub fn next(&mut self) {
        let mut next_state = HashMap::new();
        for vertex in self.circuit.appered_vertex() {
            let states: Vec<Bool> = self
                .circuit
                .get_in_edges(&vertex)
                .into_iter()
                .map(|vertex| self.state.get_index(&vertex).unwrap())
                .collect();
            let label = self.circuit.get_label(&vertex).unwrap();
            if !label.is_inlabel() {
                let next = label.next(states).unwrap();
                next_state.insert(vertex, next);
            } else {
                let this_state = self.state.get_index(&vertex).unwrap();
                next_state.insert(vertex, this_state);
            }
        }
        self.state = next_state.into();
    }
}

#[derive(Debug, Clone)]
pub struct CompositionCircuitProcess {
    left: CircuitProcess,
    left_to_right: EdgeAssign,
    right_to_left: EdgeAssign,
    right: CircuitProcess,
}

impl CompositionCircuitProcess {
    pub fn from_circuit(circuit: CompositionCircuit) -> Option<Self> {
        Some(Self {
            left: CircuitProcess::from_circuit(circuit.left())?,
            left_to_right: circuit.left_to_right_edge(),
            right_to_left: circuit.right_to_left_edge(),
            right: CircuitProcess::from_circuit(circuit.right())?,
        })
    }
    pub fn set_input(&mut self, input_state: &InputState) {
        let left_input_state: InputState = {
            let mut left_input_state: InputState = input_state.retrieve_left();
            let left_input_from_right: InputState = {
                self.right_to_left
                    .iterate()
                    .map(
                        |Edge {
                             from: r_v,
                             into: l_v,
                         }| {
                            let b: Bool = self.left.output_of_vertex(r_v).unwrap();
                            (l_v.clone(), b)
                        },
                    )
                    .into()
            };
            left_input_state.extend(left_input_from_right);
            left_input_state
        };

        let right_input_state: InputState = {
            let mut right_input_state: InputState = input_state.retrieve_right();
            let right_input_from_left: InputState = {
                self.left_to_right
                    .iterate()
                    .map(
                        |Edge {
                             from: l_v,
                             into: r_v,
                         }| {
                            let from_r_v: Bool = self.right.output_of_vertex(&r_v).unwrap();
                            (l_v.clone(), from_r_v)
                        },
                    )
                    .into()
            };
            right_input_state.extend(right_input_from_left);
            right_input_state
        };
        self.left.set_input(left_input_state);
        self.right.set_input(right_input_state);   
    }
    pub fn output(&self) -> OutputState {
        let mut map = HashMap::new();
        for (vertex, bool) in self.left.output().iterate() {
            map.insert(name_to_left_name(&vertex), bool);
        }
        for (vertex, bool) in self.right.output().iterate() {
            map.insert(name_to_right_name(&vertex), bool);
        }
        map.into()
    }
    pub fn output_of_vertex(&self, output_vertex: &VertexNumbering) -> Option<Bool> {
        if let Some(l_v) = left_name_conv_to_name(output_vertex) {
            self.left.output_of_vertex(&l_v)
        } else if let Some(r_v) = right_name_conv_to_name(output_vertex) {
            self.right.output_of_vertex(&r_v)
        } else {
            None
        }
    }
}

#[derive(Debug, Clone)]
pub struct IterationCircuitProcess {
    init_process: CircuitProcess,
    process: Vec<CircuitProcess>,
    pre_to_post: EdgeAssign,
    post_to_pre: EdgeAssign,
}

impl IterationCircuitProcess {
    pub fn from_circuit(circuit: IterationCircuit) -> Option<Self> {
        let init_process = CircuitProcess::from_circuit(circuit.iter())?;
        Some(Self {
            init_process,
            process: vec![],
            pre_to_post: circuit.pre_to_post_edge(),
            post_to_pre: circuit.post_to_pre(),
        })
    }
    pub fn set_input(&mut self, input_state: InputState) {
        let max_need = {
            let max_num_appered_in_input_state: Number = input_state
                .appered()
                .into_iter()
                .flat_map(|v| {
                    let (num, _) = iter_name_conv_to_name(&v)?;
                    Some(num)
                })
                .max()
                .unwrap_or_default();
            let now_len_of_process: Number = self.process.len().into();
            std::cmp::max(max_num_appered_in_input_state, now_len_of_process).into()
        };
        let input_states_for_each: Vec<InputState> = {
            let mut new_input_states: Vec<InputState> = (0..max_need)
                .map(|num| input_state.retrieve_iter(num.into()))
                .collect();

            for (num, process) in self.process.iter().enumerate() {
                if 0 < num {
                    for Edge { from: v1, into: v2 } in self.post_to_pre.iterate() {
                        let bool = if let Some(bool) = process.output_of_vertex(v1) {
                            bool
                        } else {
                            Bool::False
                        };
                        new_input_states[num - 1].insert(v2.clone(), bool);
                    }
                }
                for Edge { from: v1, into: v2 } in self.pre_to_post.iterate() {
                    let bool = if let Some(bool) = process.output_of_vertex(v1) {
                        bool
                    } else {
                        Bool::False
                    };
                    new_input_states[num + 1].insert(v2.clone(), bool);
                }
            }
            new_input_states
        };

        if self.process.len() < max_need {
            let need_ext = max_need - self.process.len();
            let init_process = self.init_process.clone();
            self.process.extend(vec![init_process; need_ext]);
        }

        for i in 0..max_need {
            self.process[i].set_input(input_states_for_each[i].clone());
        }
    }
    pub fn output(&self) -> OutputState {
        let mut map = HashMap::new();
        for (num, output) in self
            .process
            .iter()
            .map(|process| process.output())
            .enumerate()
        {
            map.extend(
                output
                    .iterate()
                    .iter()
                    .map(|(vertex, bool)| (name_to_iter_name(num.into(), vertex), bool.clone())),
            );
        }
        map.into()
    }
    pub fn output_of_vertex(&self, output_vertex: &VertexNumbering) -> Option<Bool> {
        let (num, vertex) = iter_name_conv_to_name(output_vertex)?;
        if self.process.len() <= num.clone().into() {
            return Some(Bool::False);
        }
        let target_process: &CircuitProcess = &self.process[num.0];
        target_process.output_of_vertex(&vertex)
    }
}

#[derive(Debug, Clone)]
pub enum CircuitProcess {
    Finite(FiniteCircuitProcess),
    Composition(Box<CompositionCircuitProcess>),
    Iteration(Box<IterationCircuitProcess>),
}

impl CircuitProcess {
    pub fn from_circuit(circuit: ExtensibleLogicCircuit) -> Option<Self> {
        unimplemented!()
    }
    pub fn set_input(&mut self, input_state: InputState) {
        unimplemented!()
    }
    pub fn output(&self) -> OutputState {
        match self {
            CircuitProcess::Finite(process) => process.output(),
            CircuitProcess::Composition(process_boxed) => process_boxed.output(),
            CircuitProcess::Iteration(process_boxed) => process_boxed.output(),
        }
    }
    pub fn output_of_vertex(&self, output_vertex: &VertexNumbering) -> Option<Bool> {
        match self {
            CircuitProcess::Finite(process) => process.output_of_vertex(output_vertex),
            CircuitProcess::Composition(process_boxed) => {
                process_boxed.output_of_vertex(output_vertex)
            }
            CircuitProcess::Iteration(process_boxed) => {
                process_boxed.output_of_vertex(output_vertex)
            }
        }
    }
    pub fn next(&mut self) {
        unimplemented!()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn fin_inout_circuit() {
        let inout: FiniteLogicCircuit = FiniteLogicCircuit::new(
            vec![("In".into(), "Out".into())].into_iter().collect(),
            vec![
                ("In".into(), (Label::input(), None)),
                ("Out".into(), (Label::output(), None)),
            ]
            .into_iter()
            .collect(),
        )
        .unwrap();
        let state: CircuitState =
            vec![("In".into(), Bool::False), ("Out".into(), Bool::True)].into();
        let mut process: FiniteCircuitProcess =
            FiniteCircuitProcess::from_circuit_and_state(inout, state).unwrap();
        process.next();
        process.output();
        process.next();
    }
    #[test]
    fn fin_and_circuit() {
        let and: FiniteLogicCircuit = FiniteLogicCircuit::new(
            vec![
                ("In1".into(), "And".into()),
                ("In2".into(), "And".into()),
                ("And".into(), "Out".into()),
            ]
            .into_iter()
            .collect(),
            vec![
                ("In1".into(), (Label::input(), None)),
                ("In2".into(), (Label::input(), None)),
                ("And".into(), (Label::and(), Some(Bool::False))),
                ("Out".into(), (Label::output(), None)),
            ]
            .into_iter()
            .collect(),
        )
        .unwrap();

        let and_state_1: InputState =
            vec![("In1".into(), Bool::True), ("In2".into(), Bool::True)].into();

        let mut process =
            FiniteCircuitProcess::from_circuit_and_input(and.clone(), and_state_1).unwrap();
        process.next();
        eprintln!("{:?}", process.output());
        process.next();
        eprintln!("{:?}", process.output());
    }

    #[test]
    fn composition() {
        // let comp_of_fin: CompositionCircuitProcess =
    }
}
