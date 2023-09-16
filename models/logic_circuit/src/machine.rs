use std::collections::{HashMap, HashSet};
use std::ops::Neg;

use utils::number::*;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum LogicLabel {
    Not,
    Or,
    And,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum InOutLabel {
    Input,
    Output,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum ControlLabel {
    Branch,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
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
    pub fn input() -> Self {
        Label::InOut(InOutLabel::Input)
    }
    pub fn output() -> Self {
        Label::InOut(InOutLabel::Output)
    }
    pub fn is_valid_inout_number(&self, input_num: Number, output_num: Number) -> bool {
        match self {
            Label::Logic(LogicLabel::Not) => input_num == 1.into() && output_num == 1.into(),
            Label::Logic(LogicLabel::And) => input_num == 2.into() && output_num == 1.into(),
            Label::Logic(LogicLabel::Or) => input_num == 2.into() && output_num == 1.into(),
            Label::InOut(InOutLabel::Input) => input_num == 0.into() && output_num == 1.into(),
            Label::InOut(InOutLabel::Output) => input_num == 1.into() && output_num == 0.into(),
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
            Label::InOut(InOutLabel::Input) => None,
            Label::InOut(InOutLabel::Output) => {
                if vec.len() == 1 {
                    Some(vec[0].clone())
                } else {
                    None
                }
            }
        }
    }
    pub fn is_inlabel(&self) -> bool {
        matches!(self, Label::InOut(InOutLabel::Input))
    }
    pub fn is_outlabel(&self) -> bool {
        matches!(self, Label::InOut(InOutLabel::Output))
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

impl From<String> for VertexNumbering {
    fn from(value: String) -> Self {
        VertexNumbering(value)
    }
}

impl From<&VertexNumbering> for String {
    fn from(value: &VertexNumbering) -> Self {
        value.0.to_owned()
    }
}

impl ToString for VertexNumbering {
    fn to_string(&self) -> String {
        self.0.to_owned()
    }
}

#[derive(Debug, Clone)]
pub struct CircuitState {
    state: HashMap<VertexNumbering, Bool>,
}

impl<T> From<T> for CircuitState
where
    T: IntoIterator<Item = (VertexNumbering, Bool)>,
{
    fn from(value: T) -> Self {
        Self {
            state: value.into_iter().collect(),
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
    fn update_with_input_state(&mut self, input: InputState) -> Option<()> {
        for (v, b) in input.0 {
            self.state.insert(v, b);
        }
        Some(())
    }
}

#[derive(Debug, Default, Clone)]
pub struct InputState(HashMap<VertexNumbering, Bool>);

impl InputState {
    pub fn appered(&self) -> HashSet<VertexNumbering> {
        self.0.keys().cloned().collect()
    }
    pub fn insert(&mut self, index: VertexNumbering, bool: Bool) {
        self.0.insert(index, bool);
    }
    pub fn get_index(&self, index: &VertexNumbering) -> Option<Bool> {
        self.0.get(index).cloned()
    }
    pub fn get_mut_index(&mut self, index: &VertexNumbering) -> Option<&mut Bool> {
        self.0.get_mut(index)
    }
    pub fn extend(&mut self, other: InputState) {
        self.0.extend(other.0);
    }
}

impl<T> From<T> for InputState
where
    T: IntoIterator<Item = (VertexNumbering, Bool)>,
{
    fn from(value: T) -> Self {
        InputState(value.into_iter().collect())
    }
}

#[derive(Debug, Default, Clone)]
pub struct OutputState(HashMap<VertexNumbering, Bool>);

impl OutputState {
    pub fn appered(&self) -> HashSet<VertexNumbering> {
        self.0.keys().cloned().collect()
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
        edges.iter().for_each(|(v1, v2)| {
            all_vertex.extend(vec![v1.clone(), v2.clone()]);
        });
        label_and_initial_state.keys().for_each(|v| {
            all_vertex.insert(v.clone());
        });

        let mut edge_appered: HashMap<VertexNumbering, bool> =
            all_vertex.iter().map(|v| (v.clone(), false)).collect();
        let mut in_edges: HashMap<VertexNumbering, HashSet<_>> = all_vertex
            .iter()
            .map(|v| (v.clone(), HashSet::new()))
            .collect();
        let mut out_edge_number: HashMap<VertexNumbering, Number> =
            all_vertex.iter().map(|v| (v.clone(), 0.into())).collect();
        for (num1, num2) in edges.iter() {
            let num = out_edge_number.get_mut(num1).unwrap();
            *num += 1.into();
            let in_set = in_edges.get_mut(num2).unwrap();
            in_set.insert(num1.clone());
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
                (Label::InOut(InOutLabel::Input), None)
                | (Label::InOut(InOutLabel::Output), None)
                | (_, Some(_)) => {}
                _ => return Err(LogicCircuitError::InValidLabelAndInitState(edgenum.clone())),
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
    pub fn appered_vertex_with_label(&self) -> HashSet<(VertexNumbering, Label)> {
        self.label_and_initial_state
            .iter()
            .map(|(v, (l, _))| (v.clone(), l.clone()))
            .collect()
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
        let op = self
            .label_and_initial_state
            .get(index)
            .map(|(label, bool)| bool.clone());
        if let Some(Some(bool)) = op {
            Some(bool)
        } else {
            None
        }
    }
    pub fn get_edge_from_label(&self, label: &Label) -> Option<VertexNumbering> {
        self.label_and_initial_state
            .iter()
            .find_map(|(edgenum, (label_edge, _))| {
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

impl EdgeAssign {
    fn new<T>(value: T) -> Self
    where
        T: IntoIterator<Item = (VertexNumbering, VertexNumbering)>,
    {
        EdgeAssign {
            in_out: value.into_iter().collect(),
        }
    }
    fn get_out_from_in(&self, v: VertexNumbering) -> HashSet<VertexNumbering> {
        self.in_out
            .iter()
            .flat_map(|(v1, v2)| if *v2 == v { Some(v1.clone()) } else { None })
            .collect()
    }
}

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
            } else if l.is_inlabel() {
                let b = input_state.get_index(v)?;
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
    pub fn new(circuit: FiniteLogicCircuit, state: CircuitState) -> Option<Self> {
        let appered_circuit = circuit.appered_vertex();
        let appered_state = state.appered();
        if appered_circuit == appered_state {
            Some(Self { circuit, state })
        } else {
            None
        }
    }
    pub fn output_from_label(&self, outputlabel: VertexNumbering) -> Option<Bool> {
        self.state.get_index(&outputlabel)
    }
    pub fn output(&self) -> HashMap<VertexNumbering, Bool> {
        self.circuit
            .appered_vertex_with_label()
            .into_iter()
            .filter_map(|(v, l)| if l.is_outlabel() { Some(v) } else { None })
            .map(|v| (v.clone(), self.state.get_index(&v).unwrap()))
            .collect()
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
    pub fn next_with_input(&mut self, input_state: InputState) -> Option<()> {
        self.state.update_with_input_state(input_state);
        Some(())
    }
}

pub struct CompositionCircuitProcess {
    left: CircuitProcess,
    left_to_right: EdgeAssign,
    right_to_left: EdgeAssign,
    right: CircuitProcess,
}

pub fn left_name_conv_to_name(vertex: &VertexNumbering) -> Option<VertexNumbering> {
    let left_start = "left-";
    if vertex.0.starts_with(left_start) {
        Some(vertex.0.split_at(left_start.len()).1.into())
    } else {
        None
    }
}

pub fn right_name_conv_to_name(vertex: &VertexNumbering) -> Option<VertexNumbering> {
    let right_start = "right-";
    if vertex.0.starts_with(right_start) {
        Some(vertex.0.split_at(right_start.len()).1.into())
    } else {
        None
    }
}

impl CompositionCircuitProcess {
    pub fn new() -> Self {
        unimplemented!()
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
    pub fn next_with_input(&mut self, input_state: InputState) -> Option<()> {
        let left_input_state: InputState = {
            let mut left_input_state: InputState = input_state
                .clone()
                .0
                .into_iter()
                .flat_map(|(v, b)| left_name_conv_to_name(&v).map(|v| (v, b)))
                .into();
            let left_input_from_right: InputState = {
                self.right_to_left
                    .in_out
                    .iter()
                    .map(|(l_v, r_v)| {
                        let from_r_v: Bool = self.left.output_of_vertex(&r_v).unwrap();
                        (l_v.clone(), from_r_v)
                    })
                    .into()
            };
            left_input_state.extend(left_input_from_right);
            left_input_state
        };

        let right_input_state: InputState = {
            let mut right_input_state: InputState = input_state
                .clone()
                .0
                .into_iter()
                .flat_map(|(v, b)| right_name_conv_to_name(&v).map(|v| (v, b)))
                .into();
            let right_input_from_left: InputState = {
                self.left_to_right
                    .in_out
                    .iter()
                    .map(|(r_v, l_v)| {
                        let from_r_v: Bool = self.right.output_of_vertex(&r_v).unwrap();
                        (l_v.clone(), from_r_v)
                    })
                    .into()
            };
            right_input_state.extend(right_input_from_left);
            right_input_state
        };
        self.left.next_with_input(left_input_state);
        self.right.next_with_input(right_input_state);
        Some(())
    }
}

pub struct IterationCircuitProcess {
    process: Vec<CircuitProcess>,
    pre_to_post: EdgeAssign,
    post_to_pre: EdgeAssign,
}

pub fn iter_name_conv_to_name(v: &VertexNumbering) -> Option<(Number, VertexNumbering)> {
    let v: Vec<_> = v.0.split('-').collect();
    if v.len() != 2 {
        return None;
    }
    let num: Number = v[0].parse::<usize>().ok()?.into();
    let vertex: VertexNumbering = v[1].into();
    Some((num, vertex))
}

impl IterationCircuitProcess {
    pub fn new() {
        unimplemented!()
    }
    pub fn output_of_vertex(&self, output_vertex: &VertexNumbering) -> Option<Bool> {
        let (num, vertex) = iter_name_conv_to_name(output_vertex)?;
        if self.process.len() <= num.clone().into() {
            return Some(Bool::False);
        }
        let target_process: &CircuitProcess = &self.process[num.0];
        target_process.output_of_vertex(&vertex)
    }
    pub fn next_with_input(&mut self, input_state: InputState) -> Option<Self> {
        let input_states: Vec<InputState> = {
            let max_num_appered_in_input_state: Number = input_state
                .appered()
                .into_iter()
                .flat_map(|v| {
                    let (num, vertex) = iter_name_conv_to_name(&v)?;
                    Some(num)
                })
                .max()
                .unwrap_or_default();
            let now_len_of_process: Number = self.process.len().into();
            let max: usize =
                std::cmp::max(max_num_appered_in_input_state, now_len_of_process).into();

            let mut new_input_states: Vec<InputState> = vec![HashMap::new().into(); max];

            for (num, vertex, bool) in input_state.0.iter().flat_map(|(v, b)| {
                iter_name_conv_to_name(v).map(|(num, vertex)| (num, vertex, b.clone()))
            }) {
                new_input_states[num.0].insert(vertex, bool);
            }

            new_input_states
        };
        None
    }
}

pub enum CircuitProcess {
    Finite(FiniteCircuitProcess),
    Composition(Box<CompositionCircuitProcess>),
    Iteration(Box<IterationCircuitProcess>),
}

impl CircuitProcess {
    pub fn output_of_vertex(&self, output_vertex: &VertexNumbering) -> Option<Bool> {
        unimplemented!()
    }
    pub fn next_with_input(&mut self, input_state: InputState) -> Option<Self> {
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
        let mut process: FiniteCircuitProcess = FiniteCircuitProcess::new(inout, state).unwrap();
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
            FiniteCircuitProcess::from_initial_state_and_input(and, and_state_1).unwrap();
        process.next();
        eprintln!("{:?}", process.output());
        process.next();
        eprintln!("{:?}", process.output());
        process.next_with_input(vec![("In1".into(), Bool::False)].into());
    }
}
