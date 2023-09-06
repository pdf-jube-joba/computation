use std::collections::{HashMap, HashSet};

use utils::number::*;

#[derive(Debug, Clone)]
pub enum LogicLabel {
    Not,
    Or,
    And,
}

#[derive(Debug, Clone)]
pub enum InOutLabel {
    Input(Number),
    Output(Number),
}

#[derive(Debug, Clone)]
pub enum ControlLabel {
    Branch,
}

#[derive(Debug, Clone)]
pub enum Label {
    Logic(LogicLabel),
    InOut(InOutLabel),
    Control(ControlLabel),
}

#[derive(Debug, Clone)]
pub enum Bool {
    True,
    False,
}

impl Bool {
    pub fn not(self) -> Self {
        match self {
            Bool::True => Bool::False,
            Bool::False => Bool::True,
        }
    }
    pub fn and(self, other: Self) -> Self {
        match (self, other) {
            (Bool::True, Bool::True) => Bool::True,
            _ => Bool::False,
        }
    }
    pub fn or(self, other: Self) -> Self {
        match (self, other) {
            (Bool::False, Bool::False) => Bool::False,
            _ => Bool::True,
        }
    }
}

pub struct LogicCircuit {
    vertex_len: Number,
    in_edge: Vec<Vec<Number>>,
    labeling: Vec<Label>,
}

pub enum LogicCircuitError {
    EdgeIsOutofRange,
    LabelIsOutofRage,
    InValidLabel(Number, Label),
}

impl LogicCircuit {
    pub fn new(
        vertex_len: Number,
        in_edge: Vec<Vec<Number>>,
        labeling: Vec<Label>,
    ) -> Result<LogicCircuit, LogicCircuitError> {
        let mut out_edge_number = vec![0; vertex_len.0];
        for (num1, num2) in in_edge.iter().enumerate() {
            let num1: Number = num1.into();
            if vertex_len <= num1 {
                return Err(LogicCircuitError::EdgeIsOutofRange);
            }
            num2.iter().for_each(|num| {
                out_edge_number[num.0] += 1;
            });
        }
        if Number::from(labeling.len()) != vertex_len {
            return Err(LogicCircuitError::LabelIsOutofRage);
        }
        for (index, label) in labeling.iter().enumerate() {
            match label {
                Label::Logic(LogicLabel::Not) => {
                    if in_edge[index].len() != 1 || out_edge_number[index] != 1 {
                        return Err(LogicCircuitError::InValidLabel(index.into(), label.clone()));
                    }
                }
                Label::Logic(LogicLabel::And) => {
                    if in_edge[index].len() != 2 || out_edge_number[index] != 1 {
                        return Err(LogicCircuitError::InValidLabel(index.into(), label.clone()));
                    }
                }
                Label::Logic(LogicLabel::Or) => {
                    if in_edge[index].len() != 2 || out_edge_number[index] != 1 {
                        return Err(LogicCircuitError::InValidLabel(index.into(), label.clone()));
                    }
                }
                Label::InOut(InOutLabel::Input(_)) => {
                    if in_edge[index].len() != 0 || out_edge_number[index] != 1 {
                        return Err(LogicCircuitError::InValidLabel(index.into(), label.clone()));
                    }
                }
                Label::InOut(InOutLabel::Output(_)) => {
                    if in_edge[index].len() != 1 || out_edge_number[index] != 0 {
                        return Err(LogicCircuitError::InValidLabel(index.into(), label.clone()));
                    }
                }
                Label::Control(ControlLabel::Branch) => {
                    if in_edge[index].len() != 1 {
                        return Err(LogicCircuitError::InValidLabel(index.into(), label.clone()));
                    }
                }
            }
        }

        Ok(LogicCircuit {
            vertex_len,
            in_edge,
            labeling,
        })
    }
    pub fn vertex_len(&self) -> Number {
        self.vertex_len.clone()
    }
}

pub struct CircuitState {
    circuit: LogicCircuit,
    state: Vec<Bool>,
}

pub enum CircuitStateError {
    Error,
}

pub fn index_by(state: &[Bool], index: Number) -> &Bool {
    &state[index.0]
}

impl CircuitState {
    pub fn new(
        circuit: LogicCircuit,
        init_state: Vec<Bool>,
    ) -> Result<CircuitState, CircuitStateError> {
        if Number::from(init_state.len()) != circuit.vertex_len() {
            return Err(CircuitStateError::Error);
        }
        Ok(Self {
            circuit,
            state: init_state,
        })
    }
    pub fn step(&mut self) {
        let mut new_state = vec![];
        for index in 0..self.circuit.vertex_len().0 {
            let label = self.circuit.labeling[index].clone();
            let in_vertexes: &[Number] = &self.circuit.in_edge[index];
            match label {
                Label::Logic(LogicLabel::Not) => {
                    let input: Bool = index_by(&self.state, in_vertexes[0].clone()).clone();
                    new_state.push(input.not());
                }
                Label::Logic(LogicLabel::And) => {
                    let input1: Bool = index_by(&self.state, in_vertexes[0].clone()).clone();
                    let input2: Bool = index_by(&self.state, in_vertexes[0].clone()).clone();
                    new_state.push(input1.and(input2));
                }
                Label::Logic(LogicLabel::Or) => {
                    let input1: Bool = index_by(&self.state, in_vertexes[0].clone()).clone();
                    let input2: Bool = index_by(&self.state, in_vertexes[0].clone()).clone();
                    new_state.push(input1.or(input2));
                }
                Label::InOut(InOutLabel::Input(_)) => {}
                Label::InOut(InOutLabel::Output(_)) => {
                    let input: Bool = index_by(&self.state, in_vertexes[0].clone()).clone();
                    new_state.push(input.clone());
                }
                Label::Control(ControlLabel::Branch) => {
                    let input: Bool = index_by(&self.state, in_vertexes[0].clone()).clone();
                    new_state.push(input.not());
                }
            }
        }
        self.state = new_state;
    }
}
