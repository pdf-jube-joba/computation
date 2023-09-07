use std::collections::{HashMap, HashSet};
use std::ops::Neg;

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
    pub fn is_valid_inout_number(&self, input_num: Number, output_num: Number) -> bool {
        match self {
            Label::Logic(LogicLabel::Not) => input_num == 1.into() && output_num == 1.into(),
            Label::Logic(LogicLabel::And) => input_num == 2.into() && output_num == 1.into(),
            Label::Logic(LogicLabel::Or) => input_num == 2.into() && output_num == 1.into(),
            Label::InOut(InOutLabel::Input(_)) => input_num == 0.into() && output_num == 1.into(),
            Label::InOut(InOutLabel::Output(_)) => input_num == 1.into() && output_num == 0.into(),
            Label::Control(ControlLabel::Branch) => input_num == 1.into(),
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

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct EdgeNumbering(String);

#[derive(Debug, Clone)]
pub struct LogicCircuit {
    input_label_len: Number,
    output_label_len: Number,
    edges: HashSet<(EdgeNumbering, EdgeNumbering)>,
    in_edges: HashMap<EdgeNumbering, HashSet<EdgeNumbering>>,
    labeling: HashMap<EdgeNumbering, Label>,
    edgenumbering_appered: HashSet<EdgeNumbering>,
}

#[derive(Debug, Clone)]
pub enum LogicCircuitError {
    EdgeIsOutofRange,
    LabelIsOutofRage,
    InValidLabel(EdgeNumbering, Label),
    LabelLacked(EdgeNumbering),
    LackOfMiddleInputNumber(Number),
    LackOfMiddleOutputNumber(Number),
}

impl LogicCircuit {
    pub fn new(
        edges: HashSet<(EdgeNumbering, EdgeNumbering)>,
        labeling: HashMap<EdgeNumbering, Label>,
    ) -> Result<LogicCircuit, LogicCircuitError> {
        let mut input_label_appered = vec![];
        let mut output_label_appered = vec![];
        let mut edgenumbering_appered = HashMap::<EdgeNumbering, bool>::new();
        let mut in_edges: HashMap<EdgeNumbering, HashSet<_>> = HashMap::new();
        let mut out_edge_number: HashMap<EdgeNumbering, Number> = HashMap::new();
        for (num1, num2) in edges.iter() {
            if let Some(num) = out_edge_number.get_mut(num1) {
                *num += 1.into();
            } else {
                out_edge_number.insert(num1.clone(), 0.into());
            }
            if let Some(set) = in_edges.get_mut(num2) {
                set.insert(num1.clone());
            } else {
                in_edges.insert(num1.clone(), HashSet::new());
            }
            edgenumbering_appered.insert(num1.clone(), false);
            edgenumbering_appered.insert(num2.clone(), false);
        }

        for (edgenum, label) in labeling.iter() {
            let edge_in_num = in_edges
                .get(edgenum)
                .ok_or(LogicCircuitError::InValidLabel(
                    edgenum.clone(),
                    label.clone(),
                ))?
                .clone()
                .len()
                .into();
            let edge_out_num = out_edge_number
                .get(edgenum)
                .ok_or(LogicCircuitError::InValidLabel(
                    edgenum.clone(),
                    label.clone(),
                ))?
                .clone();
            if !label.is_valid_inout_number(edge_in_num, edge_out_num) {
                return Err(LogicCircuitError::InValidLabel(
                    edgenum.clone(),
                    label.clone(),
                ));
            }
            edgenumbering_appered.insert(edgenum.clone(), true);
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

        for (k, v) in edgenumbering_appered.iter() {
            if !*v {
                return Err(LogicCircuitError::LabelLacked(k.clone()));
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
            input_label_len: input_label_appered.len().into(),
            output_label_len: output_label_appered.len().into(),
            edges,
            in_edges,
            labeling,
            edgenumbering_appered: edgenumbering_appered.into_iter().map(|(k, v)| k).collect(),
        })
    }
    pub fn appered_edge(&self) -> HashSet<EdgeNumbering> {
        self.edgenumbering_appered.clone()
    }
    pub fn get_label(&self, index: &EdgeNumbering) -> &Label {
        self.labeling.get(index).unwrap()
    }
    pub fn get_in_edges(&self, index: &EdgeNumbering) -> Vec<EdgeNumbering> {
        self.in_edges
            .get(index)
            .cloned()
            .unwrap()
            .into_iter()
            .collect()
    }
}

#[derive(Debug, Clone)]
pub struct CircuitState {
    state: HashMap<EdgeNumbering, Bool>,
}

impl From<HashMap<EdgeNumbering, Bool>> for CircuitState {
    fn from(value: HashMap<EdgeNumbering, Bool>) -> Self {
        Self { state: value }
    }
}

impl CircuitState {
    fn appered(&self) -> HashSet<EdgeNumbering> {
        self.state.keys().cloned().collect()
    }
    fn get_index(&mut self, index: &EdgeNumbering) -> Option<Bool> {
        self.state.get_mut(index).cloned()
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
        if circuit.appered_edge() != init_state.appered() {
            return Err(CircuitStateError::Error);
        }
        Ok(Self {
            circuit,
            state: init_state,
        })
    }
    pub fn step(&mut self) {
        let mut new_state = HashMap::<EdgeNumbering, Bool>::new();
        for index in self.circuit.appered_edge() {
            let label = self.circuit.get_label(&index);
            let in_vertexes: Vec<_> = self.circuit.get_in_edges(&index);
            match label {
                Label::Logic(LogicLabel::Not) => {
                    let input_num: EdgeNumbering = in_vertexes[0].clone();
                    let input: Bool = self.state.get_index(&input_num).unwrap();
                    new_state.insert(index, input.neg());
                }
                Label::Logic(LogicLabel::And) => {
                    let input_num1: EdgeNumbering = in_vertexes[0].clone();
                    let input_num2: EdgeNumbering = in_vertexes[1].clone();
                    let input1: Bool = self.state.get_index(&input_num1).unwrap();
                    let input2: Bool = self.state.get_index(&input_num2).unwrap();
                    new_state.insert(index, input1.and(input2));
                }
                Label::Logic(LogicLabel::Or) => {
                    let input_num1: EdgeNumbering = in_vertexes[0].clone();
                    let input_num2: EdgeNumbering = in_vertexes[1].clone();
                    let input1: Bool = self.state.get_index(&input_num1).unwrap();
                    let input2: Bool = self.state.get_index(&input_num2).unwrap();
                    new_state.insert(index, input1.or(input2));
                }
                Label::InOut(InOutLabel::Input(_)) => {}
                Label::InOut(InOutLabel::Output(_)) => {
                    let input_num: EdgeNumbering = in_vertexes[0].clone();
                    let input: Bool = self.state.get_index(&input_num).unwrap();
                    new_state.insert(index, input.clone());
                }
                Label::Control(ControlLabel::Branch) => {
                    let input_num: EdgeNumbering = in_vertexes[0].clone();
                    let input: Bool = self.state.get_index(&input_num).unwrap();
                    new_state.insert(index, input.clone());
                }
            }
        }
        self.state = new_state.into();
    }
}

#[derive(Debug, Clone)]
pub struct InOutLackedCircuitState {
    state: Vec<Option<Bool>>,
}

#[derive(Debug, Clone)]
pub struct EdgeAssign {
    max_len: Number,
    assign: Vec<Vec<Number>>,
}

impl EdgeAssign {
    pub fn new(max_len: Number, assign: Vec<Vec<Number>>) -> Option<EdgeAssign> {
        let assign_len = assign.len();
        let mut sets: HashSet<_> = HashSet::new();
        for i in 0..assign_len {
            sets.extend(&assign[i]);
            for j in i + 1..assign_len {
                if assign[i].iter().any(|elm| assign[j].contains(elm)) {
                    return None;
                }
            }
        }
        for i in 0..max_len.clone().into() {
            if !sets.contains::<Number>(&i.into()) {
                return None;
            }
        }
        Some(Self { max_len, assign })
    }
    pub fn max_len(&self) -> Number {
        self.max_len.clone()
    }
}

#[derive(Debug, Clone)]
pub struct ExtensibleLogicCircuit {
    logic_circuit: LogicCircuit,
    input_assign: EdgeAssign,
    output_assign: EdgeAssign,
    initial_state: InOutLackedCircuitState,
}

pub enum ExtensibleLogicCircuitError {
    InputLabelIndexout,
    OutputLabelIndexout,
    LengthOfStateIsDiff,
    LabelAndInitStateDiff,
}

impl ExtensibleLogicCircuit {
    pub fn new(
        logic_circuit: LogicCircuit,
        input_assign: EdgeAssign,
        output_assign: EdgeAssign,
        initial_state: InOutLackedCircuitState,
    ) -> Result<ExtensibleLogicCircuit, ExtensibleLogicCircuitError> {
        if logic_circuit.input_label_len != input_assign.max_len() {
            return Err(ExtensibleLogicCircuitError::InputLabelIndexout);
        }
        if logic_circuit.output_label_len != output_assign.max_len() {
            return Err(ExtensibleLogicCircuitError::OutputLabelIndexout);
        }

        let iter1 = &logic_circuit.labeling;
        let iter2 = &initial_state.state;

        if iter1.len() != iter2.len() {
            return Err(ExtensibleLogicCircuitError::LengthOfStateIsDiff);
        }

        for ((_, label), maybe_bool) in iter1.iter().zip(iter2) {
            match (label, maybe_bool) {
                (Label::InOut(_), None)
                | (Label::Logic(_), Some(_))
                | (Label::Control(_), Some(_)) => {
                    continue;
                }
                _ => {
                    return Err(ExtensibleLogicCircuitError::LabelAndInitStateDiff);
                }
            }
        }

        Ok(Self {
            logic_circuit,
            input_assign,
            output_assign,
            initial_state,
        })
    }
}

// pub enum CompositionError {
//     InvalidPostPre,
// }

// pub fn composition(ext1: ExtensibleLogicCircuit, ext2: ExtensibleLogicCircuit) -> Result<ExtensibleLogicCircuit,    CompositionError> {
//     let EdgeAssign { pre: pre1_in, keep: keep1_in, post: post1_in } = ext1.input_conn_num;
//     let EdgeAssign { pre: pre1_out, keep: keep1_out, post: post1_out } = ext1.output_conn_num;
//     let EdgeAssign { pre: pre2_in, keep: keep2_in, post: post2_in } = ext2.input_conn_num;
//     let EdgeAssign { pre: pre2_out, keep: keep2_out, post: post2_out } = ext2.output_conn_num;
//     if
//     todo!()
// }

// pub struct ExtensibleCircuitProcess {
//     initial_part: ExtensibleLogicCircuit,
//     iterate_part: ExtensibleLogicCircuit,
// }

// impl ExtensibleCircuitProcess {
//     pub fn new(
//         circuit: ExtensibleLogicCircuit,
//         initial_part_state: CircuitState,
//     ) -> Result<ExtensibleCircuitProcess, String> {
//         unimplemented!()
//     }
// }
