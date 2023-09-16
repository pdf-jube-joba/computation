use std::collections::{HashMap, HashSet};
use std::ops::Neg;

use utils::number::*;

#[derive(Debug, Clone, PartialEq)]
pub enum LogicLabel {
    Not,
    Or,
    And,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct InOutNumbering(String);

impl From<&str> for InOutNumbering {
    fn from(value: &str) -> Self {
        InOutNumbering(value.to_owned())
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum InOutLabel {
    Input(InOutNumbering),
    Output(InOutNumbering),
}

#[derive(Debug, Clone, PartialEq)]
pub enum ControlLabel {
    Branch,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Label {
    Logic(LogicLabel),
    InOut(InOutLabel),
    Control(ControlLabel),
}

impl Label {
    pub fn not() -> Self {
        Label::Logic(LogicLabel::Not)
    }
    pub fn and() -> Self {
        Label::Logic(LogicLabel::And)
    }
    pub fn or() -> Self {
        Label::Logic(LogicLabel::Or)
    }
    pub fn branch() -> Self {
        Label::Control(ControlLabel::Branch)
    }
    pub fn input(label: InOutNumbering) -> Self {
        Label::InOut(InOutLabel::Input(label))
    }
    pub fn output(label: InOutNumbering) -> Self {
        Label::InOut(InOutLabel::Output(label))
    }
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
    pub fn next(&self, vec: Vec<Bool>) -> Option<Bool> {
        match self {
            Label::Logic(LogicLabel::Not) => {
                if vec.len() == 1 {
                    Some(vec[0].clone().neg())
                } else {
                    None
                }
            }
            Label::Logic(LogicLabel::And) => {
                if vec.len() == 2 {
                    Some({
                        let b1 = vec[0].clone();
                        let b2 = vec[1].clone();
                        b1.and(b2)
                    })
                } else {
                    None
                }
            }
            Label::Logic(LogicLabel::Or) => {
                if vec.len() == 2 {
                    Some({
                        let b1 = vec[0].clone();
                        let b2 = vec[1].clone();
                        b1.or(b2)
                    })
                } else {
                    None
                }
            }
            Label::Control(ControlLabel::Branch) => {
                if vec.len() == 1 {
                    Some(vec[0].clone())
                } else {
                    None
                }
            }
            Label::InOut(InOutLabel::Input(_)) => {
                None
            }
            Label::InOut(InOutLabel::Output(_)) => {
                if vec.len() == 1 {
                    Some(vec[0].clone())
                } else {
                    None
                }
            }
        }
    }
    pub fn is_inlabel(&self) -> bool {
        matches!(self, Label::InOut(InOutLabel::Input(_)))
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
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
pub struct VertexNumbering(String);

impl From<&str> for VertexNumbering {
    fn from(value: &str) -> Self {
        VertexNumbering(value.to_owned())
    }
}

#[derive(Debug, Clone)]
pub struct CircuitState {
    state: HashMap<VertexNumbering, Bool>,
}

impl<T> From<T> for CircuitState where 
    T: IntoIterator<Item = (VertexNumbering, Bool)>
{
    fn from(value: T) -> Self {
        Self {
            state: value.into_iter().collect()
        }
    }
}

impl CircuitState {
    fn appered(&self) -> HashSet<VertexNumbering> {
        self.state.keys().cloned().collect()
    }
    fn get_index(&self, index: &VertexNumbering) -> Option<Bool> {
        self.state.get(index).cloned()
    }
    fn get_mut_index(&mut self, index: &VertexNumbering) -> Option<&mut Bool> {
        self.state.get_mut(index)
    }
}

#[derive(Debug, Clone)]
pub struct FiniteLogicCircuit {
    in_edges: HashMap<VertexNumbering, HashSet<VertexNumbering>>,
    label_and_initial_state: HashMap<VertexNumbering, (Label, Option<Bool>)>,
}

#[derive(Debug, Clone)]
pub enum LogicCircuitError {
    InValidLabelAndInOutNum(VertexNumbering, Label),
    InValidLabelAndInitState(VertexNumbering),
    LabelLacked(VertexNumbering),
}

impl FiniteLogicCircuit {
    pub fn new(
        edges: HashSet<(VertexNumbering, VertexNumbering)>,
        label_and_initial_state: HashMap<VertexNumbering, (Label, Option<Bool>)>,
    ) -> Result<FiniteLogicCircuit, LogicCircuitError> {
        // 計算量やばいけどめんどくさい
        let mut all_vertex = HashSet::<VertexNumbering>::new();
        edges.iter().for_each(|(v1, v2)|{
            all_vertex.extend(vec![v1.clone(), v2.clone()]);
        });
        label_and_initial_state.keys().for_each(|v|{
            all_vertex.insert(v.clone());
        });

        let mut edge_appered: HashMap<VertexNumbering, bool> = all_vertex.iter()
            .map(|v| (v.clone(), false))
            .collect();
        let mut in_edges: HashMap<VertexNumbering, HashSet<_>> = all_vertex.iter()
            .map(|v| (v.clone(), HashSet::new()))
            .collect();
        let mut out_edge_number: HashMap<VertexNumbering, Number> = all_vertex.iter()
            .map(|v| (v.clone(), 0.into()))
            .collect();
        for (num1, num2) in edges.iter() {
            let num = out_edge_number.get_mut(num1).unwrap();
            *num += 1.into();
            let in_set = in_edges.get_mut(num2).unwrap();
            in_set.insert(num1.clone());
            edge_appered.insert(num1.clone(), false);
            edge_appered.insert(num2.clone(), false);
        }
        eprintln!("{edge_appered:?} {in_edges:?} {out_edge_number:?}");

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
    pub fn appered_vertex(&self) -> HashSet<VertexNumbering> {
        self.label_and_initial_state.keys().cloned().collect()
    }
    pub fn get_label(&self, index: &VertexNumbering) -> Option<&Label> {
        self.label_and_initial_state.get(index).map(|(v, _)| v)
    }
    pub fn get_in_edges(&self, index: &VertexNumbering) -> Vec<VertexNumbering> {
        self.in_edges
            .get(index)
            .cloned()
            .unwrap()
            .into_iter()
            .collect()
    }
    pub fn get_initial_state(&self, index: &VertexNumbering) -> Option<Bool> {
        let op = self.label_and_initial_state.get(index).map(|(label, bool)| bool.clone());
        if let Some(Some(bool)) = op {
            Some(bool)
        } else {
            None
        }
    }
    pub fn get_edge_from_label(&self, label: &Label) -> Option<VertexNumbering> {
        self.label_and_initial_state.iter().find_map(|(edgenum, (label_edge, _))|{
            if *label == *label_edge {
                Some(edgenum.clone())
            } else {
                None
            }
        })
    }
}

#[derive(Debug, Clone)]
pub struct EdgeAssign {
    in_out: HashSet<(VertexNumbering, VertexNumbering)>,
}

#[derive(Debug, Clone)]
pub struct InputState(HashMap<InOutNumbering, Bool>);

// この論理回路の InOut(str) には外側からは
// InOut(format!("left-{str}")) や InOut(format!("right-{str}")) でアクセスする。 
#[derive(Debug, Clone)]
pub struct CompositionCircuit {
    left: ExtensibleLogicCircuit,
    left_to_right: EdgeAssign,
    right_to_left: EdgeAssign,
    right: ExtensibleLogicCircuit,
}

// この論理回路の InOut(str) には外側からは
// InOut(format!("{n}-{str}")) でアクセスする。 
// ただし n は初期から何番目かを指定する整数
#[derive(Debug, Clone)]
pub struct IterationCircuit {
    iter: ExtensibleLogicCircuit,
    pre_to_post: EdgeAssign,
    post_to_pre: EdgeAssign,
}

#[derive(Debug, Clone)]
pub enum ExtensibleLogicCircuit {
    FiniteCircuit(Box<FiniteLogicCircuit>),
    Composition(Box<CompositionCircuit>),
    Iteration(Box<IterationCircuit>),
}

pub struct FiniteCircuitProcess {
    circuit: FiniteLogicCircuit,
    state: CircuitState,
}

impl FiniteCircuitProcess {
    pub fn from_initial_state_and_input(
        circuit: FiniteLogicCircuit,
        input_state: InputState,
    ) -> Option<Self> {
        let mut state = HashMap::new();
        for (v, (l, s)) in circuit.label_and_initial_state.iter() {
            if let Some(b) = s {
                state.insert(v.clone(), b.clone());
            } else {
                let inout_label: InOutNumbering = match l {
                    Label::InOut(InOutLabel::Input(num)) | Label::InOut(InOutLabel::Output(num)) => {
                        num.clone()
                    } 
                    _ => {
                        return None;
                    }
                };
            }
        }
        Some(Self { circuit, state: state.into() })
    }
    pub fn new(
        circuit: FiniteLogicCircuit,
        state: CircuitState,
    ) -> Option<Self> {
        let appered_circuit = circuit.appered_vertex();
        let appered_state = state.appered();
        if appered_circuit == appered_state {
            Some(Self {
                circuit,
                state,
            })
        } else {
            None
        }
    }
    pub fn output(&self, outputlabel: InOutNumbering) -> Option<Bool> {
        let out_label: Label = Label::InOut(InOutLabel::Output(outputlabel));
        let edge: VertexNumbering = self.circuit.get_edge_from_label(&out_label)?;
        self.state.get_index(&edge)
    }
    pub fn next(&mut self) {
        let mut next_state = HashMap::new();
        for vertex in self.circuit.appered_vertex() {
            let states: Vec<Bool> = self.circuit
                .get_in_edges(&vertex)
                .into_iter()
                .map(|vertex|{
                    self.state.get_index(&vertex).unwrap()
                }).collect();
            let label = self.circuit.get_label(&vertex).unwrap();
            if label.is_inlabel() {
                let next = label.next(states).unwrap();
                next_state.insert(vertex, next);
            } else {
                let this_state = self.state.get_index(&vertex).unwrap();
                next_state.insert(vertex, this_state);
            }
        }
        self.state = next_state.into();
    }
    pub fn next_with_input(&mut self, input_state: InputState) -> Option<()> {
        unimplemented!()
    }
}

pub struct CompositionCircuitProcess {
    left: CircuitProcess,
    left_to_right: EdgeAssign,
    right_to_left: EdgeAssign,
    right: CircuitProcess,
}

pub struct IterationCircuitProcess {
    process: Vec<CircuitProcess>,
    pre_to_post: EdgeAssign,
    post_to_pre: EdgeAssign,   
}

pub enum CircuitProcess {
    Finite(FiniteCircuitProcess),
    Composition(Box<CompositionCircuitProcess>),
    Iteration(Box<IterationCircuitProcess>),
}

impl CircuitProcess {
    pub fn output(&self, outputlabel: InOutNumbering) -> Option<Bool>{
        unimplemented!()
    }
    pub fn with_input(self, input_label: InputState) -> Option<Self> {
        unimplemented!()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    pub fn fin_circuit() {
        let inout: FiniteLogicCircuit = FiniteLogicCircuit::new( 
            vec![
                ("In".into(), "Out".into())
            ].into_iter().collect(),
            vec![
                ("In".into(), (Label::input("In".into()), None)),
                ("Out".into(), (Label::output("Out".into()), None))
            ].into_iter().collect()
        ).unwrap();
        let state: CircuitState = vec![
            ("In".into(), Bool::False),
            ("Out".into(), Bool::True),
        ].into();
        let mut process: FiniteCircuitProcess = FiniteCircuitProcess::new(inout, state).unwrap();
        process.output("Out".into()).unwrap();
    }
}
