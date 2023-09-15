use std::collections::{HashMap, HashSet};
use std::ops::Neg;

use utils::number::*;

#[derive(Debug, Clone)]
pub enum LogicLabel {
    Not,
    Or,
    And,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct InOutNumbering(String);

#[derive(Debug, Clone)]
pub enum InOutLabel {
    Input(InOutNumbering),
    Output(InOutNumbering),
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

#[derive(Debug, Clone)]
pub struct FiniteLogicCircuit {
    in_edges: HashMap<EdgeNumbering, HashSet<EdgeNumbering>>,
    label_and_initial_state: HashMap<EdgeNumbering, (Label, Option<Bool>)>,
}

#[derive(Debug, Clone)]
pub enum LogicCircuitError {
    InValidLabelAndInOutNum(EdgeNumbering, Label),
    InValidLabelAndInitState(EdgeNumbering),
    LabelLacked(EdgeNumbering),
}

impl FiniteLogicCircuit {
    pub fn new(
        edges: HashSet<(EdgeNumbering, EdgeNumbering)>,
        label_and_initial_state: HashMap<EdgeNumbering, (Label, Option<Bool>)>,
    ) -> Result<FiniteLogicCircuit, LogicCircuitError> {
        let mut edge_appered = HashMap::<EdgeNumbering, bool>::new();
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
            edge_appered.insert(num1.clone(), false);
            edge_appered.insert(num2.clone(), false);
        }

        for (edgenum, (label, state)) in label_and_initial_state.iter() {
            let edge_in_num = in_edges
                .get(edgenum)
                .ok_or(LogicCircuitError::InValidLabelAndInOutNum(
                    edgenum.clone(),
                    label.clone(),
                ))?
                .clone()
                .len()
                .into();
            let edge_out_num = out_edge_number
                .get(edgenum)
                .ok_or(LogicCircuitError::InValidLabelAndInOutNum(
                    edgenum.clone(),
                    label.clone(),
                ))?
                .clone();
            if !label.is_valid_inout_number(edge_in_num, edge_out_num) {
                return Err(LogicCircuitError::InValidLabelAndInOutNum(
                    edgenum.clone(),
                    label.clone(),
                ));
            }
            edge_appered.insert(edgenum.clone(), true);
            match (label, state) {
                (Label::InOut(InOutLabel::Input(_)), None)
                | (Label::InOut(InOutLabel::Output(_)), None) 
                | (_, Some(_)) => {
                }
                _ => {
                    return Err(LogicCircuitError::InValidLabelAndInitState(edgenum.clone()))
                }
            }
        }

        for (k, v) in edge_appered.iter() {
            if !*v {
                return Err(LogicCircuitError::LabelLacked(k.clone()));
            }
        }

        Ok(FiniteLogicCircuit {
            in_edges,
            label_and_initial_state,
        })
    }
    pub fn appered_edge(&self) -> HashSet<EdgeNumbering> {
        self.label_and_initial_state.keys().cloned().collect()
    }
    pub fn get_label(&self, index: &EdgeNumbering) -> Option<&Label> {
        self.label_and_initial_state.get(index).map(|(v, _)| v)
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
pub struct EdgeAssign {
    in_out_1: HashSet<(EdgeNumbering, EdgeNumbering)>,
}

#[derive(Debug, Clone)]
pub struct Composition {
    left: ExtensibleLogicCircuit,
    left_to_right: EdgeAssign,
    right_to_left: EdgeAssign,
    right: ExtensibleLogicCircuit,
}

#[derive(Debug, Clone)]
pub struct Iteration {
    iter: ExtensibleLogicCircuit,
    pre_to_post: EdgeAssign,
    post_to_pre: EdgeAssign,
}

#[derive(Debug, Clone)]
pub enum ExtensibleLogicCircuitKind {
    FiniteCircuit(Box<FiniteLogicCircuit>),
    Composition(Box<Composition>),
    Iteration(Box<Iteration>),
}

#[derive(Debug, Clone)]
pub struct ExtensibleLogicCircuit {
    name: String,
    circuit: ExtensibleLogicCircuitKind,
}
