use std::collections::{HashMap, HashSet};
use std::ops::Neg;
use std::ops::{Index, IndexMut, Range};

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

impl Label {
    pub fn is_valid_inout_number(&self, input_num: usize, output_num: usize) -> bool {
        match self {
            Label::Logic(LogicLabel::Not) => input_num == 1 && output_num == 1,
            Label::Logic(LogicLabel::And) => input_num == 2 && output_num == 1,
            Label::Logic(LogicLabel::Or) => input_num == 2 && output_num == 1,
            Label::InOut(InOutLabel::Input(_)) => input_num == 0 && output_num == 1,
            Label::InOut(InOutLabel::Output(_)) => input_num == 1 && output_num == 0,
            Label::Control(ControlLabel::Branch) => input_num == 1,
        }
    }
}

#[derive(Debug, Clone)]
pub enum Bool {
    True,
    False,
}

impl Neg for Bool {
    type Output = Bool;
    fn neg(self) -> Self::Output {
        match self {
            Bool::True => Bool::False,
            Bool::False => Bool::True,
        }
    }
}

impl Bool {
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

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct EdgeNumbering(Number);

impl<T> Index<EdgeNumbering> for Vec<T> {
    type Output = T;
    fn index(&self, index: EdgeNumbering) -> &Self::Output {
        &self[index.0 .0]
    }
}

impl<T> IndexMut<EdgeNumbering> for Vec<T> {
    fn index_mut(&mut self, index: EdgeNumbering) -> &mut Self::Output {
        &mut self[index.0 .0]
    }
}

impl From<EdgeNumbering> for usize {
    fn from(value: EdgeNumbering) -> Self {
        value.0 .0
    }
}

impl From<usize> for EdgeNumbering {
    fn from(value: usize) -> Self {
        EdgeNumbering(Number(value))
    }
}

#[derive(Debug, Clone)]
pub struct LogicCircuit {
    vertex_len: EdgeNumbering,
    input_label_len: Number,
    output_label_len: Number,
    in_edge: Vec<Vec<EdgeNumbering>>,
    labeling: Vec<Label>,
}

#[derive(Debug, Clone)]
pub enum LogicCircuitError {
    EdgeIsOutofRange,
    LabelIsOutofRage,
    InValidLabel(Number, Label),
    LackOfMiddleInputNumber(Number),
    LackOfMiddleOutputNumber(Number),
}

impl LogicCircuit {
    pub fn new(
        vertex_len: EdgeNumbering,
        in_edge: Vec<Vec<EdgeNumbering>>,
        labeling: Vec<Label>,
    ) -> Result<LogicCircuit, LogicCircuitError> {
        let mut out_edge_number: Vec<usize> = vec![0; vertex_len.clone().into()];
        let mut input_label_appered = vec![];
        let mut output_label_appered = vec![];
        for (num1, num2) in in_edge.iter().enumerate() {
            let num1: EdgeNumbering = num1.into();
            if vertex_len <= num1 {
                return Err(LogicCircuitError::EdgeIsOutofRange);
            }
            num2.iter().for_each(|num| {
                out_edge_number[num.clone()] += 1;
            });
        }
        if EdgeNumbering::from(labeling.len()) != vertex_len {
            return Err(LogicCircuitError::LabelIsOutofRage);
        }
        for (index, label) in labeling.iter().enumerate() {
            let edge_in_num = in_edge[index].len();
            let edge_out_num = out_edge_number[index];
            if !label.is_valid_inout_number(edge_in_num, edge_out_num) {
                return Err(LogicCircuitError::InValidLabel(index.into(), label.clone()));
            }
            match label {
                Label::InOut(InOutLabel::Input(num)) => {
                    input_label_appered.push(num.clone());
                }
                Label::InOut(InOutLabel::Output(num)) => {
                    output_label_appered.push(num.clone());
                }
                _ => {}
            }
        }

        input_label_appered.sort();
        for (index, num) in input_label_appered.iter().enumerate() {
            if Number(index) != *num {
                return Err(LogicCircuitError::LackOfMiddleInputNumber(index.into()));
            }
        }

        output_label_appered.sort();
        for (index, num) in output_label_appered.iter().enumerate() {
            if Number(index) != *num {
                return Err(LogicCircuitError::LackOfMiddleOutputNumber(index.into()));
            }
        }

        Ok(LogicCircuit {
            vertex_len,
            input_label_len: input_label_appered.len().into(),
            output_label_len: output_label_appered.len().into(),
            in_edge,
            labeling,
        })
    }
    pub fn vertex_len(&self) -> EdgeNumbering {
        self.vertex_len.clone()
    }
}

#[derive(Debug, Clone)]
pub struct CircuitState {
    state: Vec<Bool>,
}

impl From<Vec<Bool>> for CircuitState {
    fn from(value: Vec<Bool>) -> Self {
        Self { state: value }
    }
}

impl CircuitState {
    fn state_len(&self) -> EdgeNumbering {
        self.state.len().into()
    }
    fn index(&self, index: EdgeNumbering) -> Bool {
        self.state[index].clone()
    }
}

pub struct CircuitProcess {
    circuit: LogicCircuit,
    state: CircuitState,
}

pub enum CircuitStateError {
    Error,
}

impl CircuitProcess {
    pub fn new(
        circuit: LogicCircuit,
        init_state: CircuitState,
    ) -> Result<CircuitProcess, CircuitStateError> {
        if init_state.state_len() != circuit.vertex_len() {
            return Err(CircuitStateError::Error);
        }
        Ok(Self {
            circuit,
            state: init_state,
        })
    }
    pub fn step(&mut self) {
        let mut new_state = vec![];
        for index in 0..self.circuit.vertex_len().into() {
            let label = self.circuit.labeling[index].clone();
            let in_vertexes: &[EdgeNumbering] = &self.circuit.in_edge[index];
            match label {
                Label::Logic(LogicLabel::Not) => {
                    let input_num: EdgeNumbering = in_vertexes[0].clone();
                    let input: Bool = self.state.index(input_num);
                    new_state.push(input.neg());
                }
                Label::Logic(LogicLabel::And) => {
                    let input_num1: EdgeNumbering = in_vertexes[0].clone();
                    let input_num2: EdgeNumbering = in_vertexes[1].clone();
                    let input1: Bool = self.state.index(input_num1);
                    let input2: Bool = self.state.index(input_num2);
                    new_state.push(input1.and(input2));
                }
                Label::Logic(LogicLabel::Or) => {
                    let input_num1: EdgeNumbering = in_vertexes[0].clone();
                    let input_num2: EdgeNumbering = in_vertexes[1].clone();
                    let input1: Bool = self.state.index(input_num1);
                    let input2: Bool = self.state.index(input_num2);
                    new_state.push(input1.or(input2));
                }
                Label::InOut(InOutLabel::Input(_)) => {}
                Label::InOut(InOutLabel::Output(_)) => {
                    let input_num: EdgeNumbering = in_vertexes[0].clone();
                    let input: Bool = self.state.index(input_num);
                    new_state.push(input.clone());
                }
                Label::Control(ControlLabel::Branch) => {
                    let input_num: EdgeNumbering = in_vertexes[0].clone();
                    let input: Bool = self.state.index(input_num);
                    new_state.push(input.clone());
                }
            }
        }
        self.state = new_state.into();
    }
}

#[derive(Debug, Clone)]
pub struct ExtensibleLogicCircuit {
    initial_part: LogicCircuit,
    initial_part_input_len: Range<EdgeNumbering>,
    initial_part_output_len: Range<EdgeNumbering>,
    extension_part: LogicCircuit,
    extension_part_pre_input_len: Number,
    extension_part_pre_output_len: Number,
    extension_part_post_input_len: Number,
    extension_part_post_output_len: Number,
}

pub enum ExtensibleLogicCircuitError {
    InitialParInputLabelIndexout,
}

impl ExtensibleLogicCircuit {
    pub fn new(
        initial_part: LogicCircuit,
        initial_part_input_len: Number,
        initial_part_output_len: Number,
        extension_part: LogicCircuit,
        extension_part_pre_input_len: Number,
        extension_part_pre_output_len: Number,
        extension_part_post_input_len: Number,
        extension_part_post_output_len: Number,
    ) -> Result<ExtensibleLogicCircuit, ExtensibleLogicCircuitError> {
        if initial_part.input_label_len < initial_part_input_len {
            return Err(ExtensibleLogicCircuitError::InitialParInputLabelIndexout);
        }
        if initial_part.output_label_len < initial_part_output_len {
            return Err(ExtensibleLogicCircuitError::InitialParInputLabelIndexout);
        }
        // if extension_part.input_label_len <
        unimplemented!()
    }
}

pub struct ExtensibleCircuitProcess {
    circuit: ExtensibleLogicCircuit,
    initial_part_state: CircuitState,
    extension_parts_states: Vec<CircuitState>,
}
