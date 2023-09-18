use std::collections::{HashMap, HashSet};

pub mod circuit_components;
use circuit_components::*;

pub mod logic_circuit;
use logic_circuit::*;

#[derive(Debug, Clone)]
pub enum ProcessGeneralError {
    VertexNumberingIsOutofIndex,
    InputStatePointInvalidLabel,
    InputStatePointInvalidVertex,
    PointedVertexIsNotOutput,
}

trait CircuitProcessInterface {
    fn set_input(&mut self, input_state: InputState) -> Result<(), ProcessGeneralError>;
    fn output(&self) -> OutputState;
    fn output_of_vertex(&self, vertex: &VertexNumbering) -> Result<Bool, ProcessGeneralError>;
    fn next(&mut self);
}

#[derive(Debug, Clone)]
pub struct FiniteCircuitProcess {
    circuit: FiniteLogicCircuit,
    state: CircuitState,
}

impl From<FiniteLogicCircuit> for FiniteCircuitProcess {
    fn from(circuit: FiniteLogicCircuit) -> Self {
        let mut init_state = HashMap::new();
        for (v, _l, s) in circuit.iterate_as_set() {
            if let Some(b) = s {
                init_state.insert(v.clone(), b.clone());
            } else {
                init_state.insert(v.clone(), Bool::False);
            }
        }
        Self {
            circuit,
            state: init_state.into(),
        }
    }
}

impl CircuitProcessInterface for FiniteCircuitProcess {
    // input_state should exactly point a input label in circuit
    fn set_input(&mut self, input_state: InputState) -> Result<(), ProcessGeneralError> {
        for (v, b) in input_state.clone().iterate() {
            let label = self
                .circuit
                .get_label(&v)
                .ok_or(ProcessGeneralError::VertexNumberingIsOutofIndex)?;
            if !label.is_inlabel() {
                return Err(ProcessGeneralError::InputStatePointInvalidLabel);
            }
            self
                .state
                .set_index(v.clone(), b);
        }
        Ok(())
    }
    fn output(&self) -> OutputState {
        self.circuit
            .appered_vertex_with_label()
            .into_iter()
            .filter_map(|(v, l)| if l.is_outlabel() { Some(v) } else { None })
            .map(|v| (v.clone(), self.state.get_index(&v)))
            .into()
    }
    fn output_of_vertex(&self, outputlabel: &VertexNumbering) -> Result<Bool, ProcessGeneralError> {
        let label = self
            .circuit
            .get_label(outputlabel)
            .ok_or(ProcessGeneralError::VertexNumberingIsOutofIndex)?;
        if !label.is_outlabel() {
            return Err(ProcessGeneralError::PointedVertexIsNotOutput);
        }
        Ok(self.state.get_index(outputlabel))
    }
    fn next(&mut self) {
        let mut next_state = HashMap::new();
        for vertex in self.circuit.appered_vertex() {
            let states: Vec<Bool> = self
                .circuit
                .get_in_edges(&vertex)
                .into_iter()
                .map(|vertex| self.state.get_index(&vertex))
                .collect();
            let label = self.circuit.get_label(&vertex).unwrap();
            if !label.is_inlabel() {
                let next = label.next(states).unwrap();
                next_state.insert(vertex, next);
            } else {
                let this_state = self.state.get_index(&vertex);
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
    fn align_inout_of_left_right(&mut self) {
        let left_input_from_right: InputState = {
            let right_output = self.right.output();
            output_to_input_with_edge_assign(right_output, self.right_to_left.clone())
        };
        let right_input_from_left: InputState = {
            let left_output = self.left.output();
            output_to_input_with_edge_assign(left_output, self.left_to_right.clone())
        };
        self.left.set_input(left_input_from_right).unwrap();
        self.right.set_input(right_input_from_left).unwrap();
    }
}

impl From<CompositionCircuit> for CompositionCircuitProcess {
    fn from(circuit: CompositionCircuit) -> Self {
        Self {
            left: CircuitProcess::from(circuit.left()),
            left_to_right: circuit.left_to_right_edge(),
            right_to_left: circuit.right_to_left_edge(),
            right: CircuitProcess::from(circuit.right()),
        }
    }
}

impl CircuitProcessInterface for CompositionCircuitProcess {
    fn set_input(&mut self, input_state: InputState) -> Result<(), ProcessGeneralError> {
        let left_input_state: InputState = {
            let left = input_state.retrieve_left();
            for v in left.appered_as_true() {
                if self.right_to_left.contains_as_into(&v) {
                    return Err(ProcessGeneralError::InputStatePointInvalidVertex);
                }
            }
            left
        };
        let right_input_state: InputState = {
            let right = input_state.retrieve_right();
            for v in right.appered_as_true() {
                if self.left_to_right.contains_as_into(&v) {
                    return Err(ProcessGeneralError::InputStatePointInvalidVertex);
                }
            }
            right
        };
        self.left.set_input(left_input_state)?;
        self.right.set_input(right_input_state)?;
        Ok(())
    }
    fn output(&self) -> OutputState {
        let mut map = HashMap::new();
        for (vertex, bool) in self.left.output().iterate() {
            map.insert(name_to_left_name(&vertex), bool);
        }
        for (vertex, bool) in self.right.output().iterate() {
            map.insert(name_to_right_name(&vertex), bool);
        }
        map.into()
    }
    fn output_of_vertex(
        &self,
        output_vertex: &VertexNumbering,
    ) -> Result<Bool, ProcessGeneralError> {
        if let Some(l_v) = left_name_conv_to_name(output_vertex) {
            self.left.output_of_vertex(&l_v)
        } else if let Some(r_v) = right_name_conv_to_name(output_vertex) {
            self.right.output_of_vertex(&r_v)
        } else {
            Err(ProcessGeneralError::PointedVertexIsNotOutput)
        }
    }
    fn next(&mut self) {
        self.align_inout_of_left_right();
        self.left.next();
        self.right.next();
        self.align_inout_of_left_right();
    }
}

#[derive(Debug, Clone)]
pub struct IterationCircuitProcess {
    init_process: CircuitProcess,
    process: Vec<CircuitProcess>,
    pre_to_post: EdgeAssign,
    post_to_pre: EdgeAssign,
}

impl From<IterationCircuit> for IterationCircuitProcess {
    fn from(circuit: IterationCircuit) -> Self {
        let init_process = CircuitProcess::from(circuit.iter());
        Self {
            init_process,
            process: vec![],
            pre_to_post: circuit.pre_to_post_edge(),
            post_to_pre: circuit.post_to_pre(),
        }
    }
}

impl IterationCircuitProcess {
    fn align_iter(&mut self) {
        let len_of_all = self.process.len();
        let output_states: Vec<_> = self
            .process
            .iter()
            .map(|process| process.output())
            .collect();
        for (i, output_state) in output_states.iter().enumerate() {
            if i != 0 {
                let pre_input_from_post = output_to_input_with_edge_assign(
                    output_state.clone(),
                    self.post_to_pre.clone(),
                );
                self.process[i - 1].set_input(pre_input_from_post).unwrap();
            }
            let post_input_from_pre =
                output_to_input_with_edge_assign(output_state.clone(), self.pre_to_post.clone());
            if i + 1 < len_of_all {
                self.process[i + 1].set_input(post_input_from_pre).unwrap();
            } else if !post_input_from_pre.appered_as_true().is_empty() {
                self.process.push(self.init_process.clone());
                self.process[len_of_all]
                    .set_input(post_input_from_pre)
                    .unwrap();
            }
        }
    }
}

impl CircuitProcessInterface for IterationCircuitProcess {
    fn set_input(&mut self, input_state: InputState) -> Result<(), ProcessGeneralError> {
        let input_states = input_state.retrieve_iter_vec();
        if self.process.len() <= input_states.len() {
            let init_process = self.init_process.clone();
            self.process
                .extend(vec![init_process; input_states.len() - self.process.len()]);
        }
        for i in 0..input_states.len() {
            self.process[i].set_input(input_states[i].clone())?;
        }
        Ok(())
    }
    fn output(&self) -> OutputState {
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
    fn output_of_vertex(
        &self,
        output_vertex: &VertexNumbering,
    ) -> Result<Bool, ProcessGeneralError> {
        let (num, vertex) = iter_name_conv_to_name(output_vertex)
            .ok_or(ProcessGeneralError::PointedVertexIsNotOutput)?;
        if self.process.len() <= num.clone().into() {
            return Err(ProcessGeneralError::VertexNumberingIsOutofIndex);
        }
        let target_process: &CircuitProcess = &self.process[num.0];
        target_process.output_of_vertex(&vertex)
    }
    fn next(&mut self) {
        self.align_iter();
        self.process.iter_mut().for_each(|process| {
            process.next();
        });
        self.align_iter();
    }
}

#[derive(Debug, Clone)]
pub enum CircuitProcess {
    Finite(FiniteCircuitProcess),
    Composition(Box<CompositionCircuitProcess>),
    Iteration(Box<IterationCircuitProcess>),
}

impl From<ExtensibleLogicCircuit> for CircuitProcess {
    fn from(value: ExtensibleLogicCircuit) -> Self {
        match value {
            ExtensibleLogicCircuit::FiniteCircuit(circuit) => {
                CircuitProcess::Finite(FiniteCircuitProcess::from(*circuit))
            }
            ExtensibleLogicCircuit::Composition(circuit) => {
                CircuitProcess::Composition(Box::new(CompositionCircuitProcess::from(*circuit)))
            }
            ExtensibleLogicCircuit::Iteration(circuit) => {
                CircuitProcess::Iteration(Box::new(IterationCircuitProcess::from(*circuit)))
            }

        }
    }
}

impl CircuitProcessInterface for CircuitProcess {
    fn set_input(&mut self, input_state: InputState) -> Result<(), ProcessGeneralError> {
        match self {
            CircuitProcess::Finite(circuit) => circuit.set_input(input_state),
            CircuitProcess::Composition(circuit) => circuit.set_input(input_state),
            CircuitProcess::Iteration(circuit) => circuit.set_input(input_state),
        }
    }
    fn output(&self) -> OutputState {
        match self {
            CircuitProcess::Finite(process) => process.output(),
            CircuitProcess::Composition(process_boxed) => process_boxed.output(),
            CircuitProcess::Iteration(process_boxed) => process_boxed.output(),
        }
    }
    fn output_of_vertex(
        &self,
        output_vertex: &VertexNumbering,
    ) -> Result<Bool, ProcessGeneralError> {
        match self {
            CircuitProcess::Finite(process) => process.output_of_vertex(output_vertex),
            CircuitProcess::Composition(process_boxed) => process_boxed.output_of_vertex(output_vertex),
            CircuitProcess::Iteration(process_boxed) => process_boxed.output_of_vertex(output_vertex),
        }
    }
    fn next(&mut self) {
        match self {
            CircuitProcess::Finite(process) => process.next(),
            CircuitProcess::Composition(process_boxed) => process_boxed.next(),
            CircuitProcess::Iteration(process_boxed) => process_boxed.next(),
        }
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
        let mut process: FiniteCircuitProcess = FiniteCircuitProcess::from(inout.clone());
        let input: InputState = vec![("In".into(), Bool::True)].into();
        process.set_input(input.clone()).unwrap();
        process.next();
        process.output();
        process.next();

        let mut inout_process: CircuitProcess = CircuitProcess::from(ExtensibleLogicCircuit::from(inout));
        inout_process.set_input(input).unwrap();
        inout_process.next();
        inout_process.output();
        inout_process.next();

    }
    // #[test]
    // fn fin_and_circuit() {
    //     let and: FiniteLogicCircuit = FiniteLogicCircuit::new(
    //         vec![
    //             ("In1".into(), "And".into()),
    //             ("In2".into(), "And".into()),
    //             ("And".into(), "Out".into()),
    //         ]
    //         .into_iter()
    //         .collect(),
    //         vec![
    //             ("In1".into(), (Label::input(), None)),
    //             ("In2".into(), (Label::input(), None)),
    //             ("And".into(), (Label::and(), Some(Bool::False))),
    //             ("Out".into(), (Label::output(), None)),
    //         ]
    //         .into_iter()
    //         .collect(),
    //     )
    //     .unwrap();

    //     let and_state_1: InputState =
    //         vec![("In1".into(), Bool::True), ("In2".into(), Bool::True)].into();

    //     let mut process =
    //         FiniteCircuitProcess::from_circuit_and_input(and.clone(), and_state_1).unwrap();
    //     process.next();
    //     eprintln!("{:?}", process.output());
    //     process.next();
    //     eprintln!("{:?}", process.output());
    // }

    // #[test]
    // fn composition() {
    //     // let comp_of_fin: CompositionCircuitProcess =
    // }
}
