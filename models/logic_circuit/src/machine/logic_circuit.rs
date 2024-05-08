use super::circuit_components::*;
use std::{
    clone,
    collections::{HashMap, HashSet},
    fmt::Display,
    ops::Neg,
};
use utils::number::*;

type InPin = String;
type OtPin = String;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Gate {
    Not {
        state: Bool,
        input: Bool,
    },
    And {
        state: Bool,
        input0: Bool,
        input1: Bool,
    },
    Or {
        state: Bool,
        input0: Bool,
        input1: Bool,
    },
    Cst0 {},
    Br {
        state: Bool,
        input: Bool,
    },
}

impl Gate {
    fn new_not(state: Bool) -> Self {
        Gate::Not {
            state,
            input: Bool::False,
        }
    }
    fn state(&self) -> &Bool {
        match self {
            Gate::Not { state, input } => state,
            _ => unreachable!(),
        }
    }
    fn getmut_input(&mut self, input_name: &str) -> Option<&mut Bool> {
        match (self, input_name) {
            (Gate::Not { state, input }, "IN") => Some(input),
            _ => None,
        }
    }
    fn get_output(&self, output_name: &str) -> Option<&Bool> {
        match (self, output_name) {
            (Gate::Not { state, input }, "OUT") => Some(input),
            _ => None,
        }
    }
    fn next(&mut self) {
        match self {
            Gate::Not { state, input } => {
                *state = input.neg();
            }
            _ => {}
        }
    }
}

pub struct Path(Vec<String>);

impl Path {
    pub fn from_str(path: String) -> Self {
        Path(path.split(".").map(|s| s.to_string()).collect())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]

enum LoC {
    Gate {
        name: String,
        gate: Gate,
    },
    FinGraph {
        name: String,
        set_of_lc: HashMap<String, LoC>,
        edges: HashSet<((String, String), (String, String))>,
    },
    Iter {
        name: String,
        lc_init: Box<LoC>,
        lc_extended: Vec<LoC>,
        next_edges: HashSet<(String, String)>,
        prev_edges: HashSet<(String, String)>,
    },
}

impl LoC {
    fn new_gate(name: String, gate: Gate) -> Self {
        LoC::Gate { name, gate }
    }
    fn new_fingraph(
        name: String,
        set_of_lc: Vec<(String, LoC)>,
        edges: Vec<((String, String), (String, String))>,
    ) -> Self {
        todo!()
    }
    fn name(&self) -> String {
        match self {
            LoC::Gate { name, gate } => name.clone(),
            LoC::FinGraph {
                name,
                set_of_lc,
                edges,
            } => name.clone(),
            LoC::Iter {
                name,
                lc_init,
                lc_extended,
                next_edges,
                prev_edges,
            } => name.clone(),
        }
    }
    fn new_iter() -> Self {
        todo!()
    }
    fn peek_lc_in_this(&self, name_loc: &str) -> Option<&LoC> {
        match self {
            LoC::Gate { name, gate } => None,
            LoC::FinGraph {
                name,
                set_of_lc,
                edges,
            } => set_of_lc
                .iter()
                .find_map(|(s, lc)| if name_loc == s { Some(lc) } else { None }),
            _ => {
                unimplemented!()
            }
        }
    }
    fn peek_gate_from_path(&self, path: &[&str]) -> Option<&Gate> {
        if path.is_empty() {
            match self {
                LoC::Gate { name, gate } => Some(gate),
                _ => None,
            }
        } else {
            let lc = self.peek_lc_in_this(path[0])?;
            lc.peek_gate_from_path(&path[1..])
        }
    }
    fn getmut_this_input(&mut self, input_name: &str) -> Option<&mut Bool> {
        todo!()
    }
    fn get_this_output(&self, output_name: &str) -> Option<&Bool> {
        todo!()
    }
    fn next(&mut self) {
        todo!()
    }
}

#[derive(Debug, Clone)]
pub enum LogicCircuitError {
    InValidLabelAndInOutNum(VertexNumbering, Label),
    InValidLabelAndInitState(VertexNumbering),
    LabelLacked(VertexNumbering),
    EdgeAssignIsOutofIndex(VertexNumbering),
    EdgeAssignInvalid(VertexNumbering),
    EdgeAssignIsConflict,
}

#[derive(Debug, Clone)]
pub struct FiniteLogicCircuit {
    in_edges: HashMap<VertexNumbering, HashSet<VertexNumbering>>,
    label_and_initial_state: HashMap<VertexNumbering, (Label, Option<Bool>)>,
}

impl FiniteLogicCircuit {
    pub fn new<T1, T2>(
        edges: T1,
        label_and_initial_state: T2,
    ) -> Result<FiniteLogicCircuit, LogicCircuitError>
    where
        T1: IntoIterator<Item = (VertexNumbering, VertexNumbering)>,
        T2: IntoIterator<Item = (VertexNumbering, (Label, Option<Bool>))>,
    {
        // 計算量やばいけどめんどくさい

        let edges: HashSet<(VertexNumbering, VertexNumbering)> = edges.into_iter().collect();
        let label_and_initial_state: HashMap<VertexNumbering, (Label, Option<Bool>)> =
            label_and_initial_state.into_iter().collect();

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
    pub fn is_appered(&self, index: &VertexNumbering) -> bool {
        self.label_and_initial_state.keys().any(|v| *v == *index)
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
            .map(|(_, bool)| bool.clone());
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
    pub fn iterate_as_set(
        &self,
    ) -> impl Iterator<Item = (&VertexNumbering, &Label, &Option<Bool>)> {
        self.label_and_initial_state
            .iter()
            .map(|(v, (l, b))| (v, l, b))
    }
}

impl Display for FiniteLogicCircuit {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut string = String::new();
        for v in self.appered_vertex() {
            let label_string = self.get_label(&v).unwrap();
            let in_edge = self.get_in_edges(&v);
            string.push_str(&format!(
                "{v}: label: {label_string} in_edge: {in_edge:?}\n"
            ));
        }
        write!(f, "")
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

impl CompositionCircuit {
    pub fn new(
        left: ExtensibleLogicCircuit,
        left_to_right: EdgeAssign,
        right_to_left: EdgeAssign,
        right: ExtensibleLogicCircuit,
    ) -> Result<Self, LogicCircuitError> {
        for (from, into) in left_to_right.iterate_over_v() {
            let label = left
                .get_label(from)
                .ok_or(LogicCircuitError::EdgeAssignIsOutofIndex(from.clone()))?;
            if !label.is_outlabel() {
                return Err(LogicCircuitError::EdgeAssignInvalid(from.clone()));
            }
            let label = right
                .get_label(into)
                .ok_or(LogicCircuitError::EdgeAssignIsOutofIndex(into.clone()))?;
            if !label.is_inlabel() {
                return Err(LogicCircuitError::EdgeAssignInvalid(into.clone()));
            }
        }
        for (from, into) in right_to_left.iterate_over_v() {
            let label = right
                .get_label(from)
                .ok_or(LogicCircuitError::EdgeAssignIsOutofIndex(from.clone()))?;
            if !label.is_outlabel() {
                return Err(LogicCircuitError::EdgeAssignInvalid(from.clone()));
            }
            let label = left
                .get_label(into)
                .ok_or(LogicCircuitError::EdgeAssignIsOutofIndex(into.clone()))?;
            if !label.is_inlabel() {
                return Err(LogicCircuitError::EdgeAssignInvalid(into.clone()));
            }
        }
        Ok(Self {
            left,
            left_to_right,
            right_to_left,
            right,
        })
    }
    pub fn left(&self) -> ExtensibleLogicCircuit {
        self.left.clone()
    }
    pub fn right(&self) -> ExtensibleLogicCircuit {
        self.right.clone()
    }
    pub fn left_to_right_edge(&self) -> EdgeAssign {
        self.left_to_right.clone()
    }
    pub fn right_to_left_edge(&self) -> EdgeAssign {
        self.right_to_left.clone()
    }
    pub fn get_label(&self, index: &VertexNumbering) -> Option<&Label> {
        if let Some(index) = left_name_conv_to_name(index) {
            self.left.get_label(&index)
        } else if let Some(index) = right_name_conv_to_name(index) {
            self.right.get_label(&index)
        } else {
            None
        }
    }
}

impl Display for CompositionCircuit {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut string = String::new();

        string.push_str("left:\n");
        string.push_str(&indent(self.left.to_string()));
        string.push_str("right:\n");
        string.push_str(&indent(self.right.to_string()));

        for (from, into) in self.left_to_right.iterate_over_v() {
            string.push_str(&format!("l-r: {from} -> {into}"));
        }
        for (from, into) in self.right_to_left.iterate_over_v() {
            string.push_str(&format!("r-l: {into} <- {from}"));
        }

        write!(f, "{string}")
    }
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

impl From<(ExtensibleLogicCircuit, EdgeAssign, EdgeAssign)> for IterationCircuit {
    fn from(value: (ExtensibleLogicCircuit, EdgeAssign, EdgeAssign)) -> Self {
        Self {
            iter: value.0,
            pre_to_post: value.1,
            post_to_pre: value.2,
        }
    }
}

impl IterationCircuit {
    pub fn new(
        iter: ExtensibleLogicCircuit,
        pre_to_post: EdgeAssign,
        post_to_pre: EdgeAssign,
    ) -> Result<Self, LogicCircuitError> {
        let mut out_appered = HashSet::new();
        let mut in_appered = HashSet::new();

        for (from, into) in pre_to_post.iterate_over_v() {
            let label = iter
                .get_label(from)
                .ok_or(LogicCircuitError::EdgeAssignIsOutofIndex(from.clone()))?;
            if !label.is_outlabel() {
                return Err(LogicCircuitError::EdgeAssignInvalid(from.clone()));
            }
            let label = iter
                .get_label(into)
                .ok_or(LogicCircuitError::EdgeAssignIsOutofIndex(into.clone()))?;
            if !label.is_inlabel() {
                return Err(LogicCircuitError::EdgeAssignInvalid(into.clone()));
            }
            out_appered.insert(from);
            in_appered.insert(into);
        }
        for (from, into) in post_to_pre.iterate_over_v() {
            let label = iter
                .get_label(from)
                .ok_or(LogicCircuitError::EdgeAssignIsOutofIndex(from.clone()))?;
            if !label.is_outlabel() {
                return Err(LogicCircuitError::EdgeAssignInvalid(from.clone()));
            }
            let label = iter
                .get_label(into)
                .ok_or(LogicCircuitError::EdgeAssignIsOutofIndex(into.clone()))?;
            if !label.is_inlabel() {
                return Err(LogicCircuitError::EdgeAssignInvalid(into.clone()));
            }
            if out_appered.contains(from) || in_appered.contains(into) {
                return Err(LogicCircuitError::EdgeAssignIsConflict);
            }
        }
        Ok(Self {
            iter,
            pre_to_post,
            post_to_pre,
        })
    }
    pub fn iter(&self) -> ExtensibleLogicCircuit {
        self.iter.clone()
    }
    pub fn pre_to_post_edge(&self) -> EdgeAssign {
        self.pre_to_post.clone()
    }
    pub fn post_to_pre(&self) -> EdgeAssign {
        self.post_to_pre.clone()
    }
    pub fn get_label(&self, index: &VertexNumbering) -> Option<&Label> {
        let (_, index) = iter_name_conv_to_name(index)?;
        self.iter.get_label(&index)
    }
}

impl Display for IterationCircuit {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut string = String::new();
        string.push_str("iter:\n");
        string.push_str(&self.iter.to_string());
        for (from, into) in self.pre_to_post.iterate_over_v() {
            string.push_str(&format!("l-r: {from} -> {into}"));
        }
        for (from, into) in self.post_to_pre.iterate_over_v() {
            string.push_str(&format!("r-l: {into} <- {from}"));
        }
        write!(f, "{string}")
    }
}

#[derive(Debug, Clone)]
pub enum ExtensibleLogicCircuit {
    FiniteCircuit(Box<FiniteLogicCircuit>),
    Composition(Box<CompositionCircuit>),
    Iteration(Box<IterationCircuit>),
}

impl From<FiniteLogicCircuit> for ExtensibleLogicCircuit {
    fn from(value: FiniteLogicCircuit) -> Self {
        Self::FiniteCircuit(Box::new(value))
    }
}

impl From<CompositionCircuit> for ExtensibleLogicCircuit {
    fn from(value: CompositionCircuit) -> Self {
        Self::Composition(Box::new(value))
    }
}

impl From<IterationCircuit> for ExtensibleLogicCircuit {
    fn from(value: IterationCircuit) -> Self {
        Self::Iteration(Box::new(value))
    }
}

impl ExtensibleLogicCircuit {
    pub fn get_label(&self, index: &VertexNumbering) -> Option<&Label> {
        match self {
            ExtensibleLogicCircuit::FiniteCircuit(circuit) => circuit.get_label(index),
            ExtensibleLogicCircuit::Composition(circuit) => circuit.get_label(index),
            ExtensibleLogicCircuit::Iteration(circuit) => circuit.get_label(index),
        }
    }
}

impl Display for ExtensibleLogicCircuit {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ExtensibleLogicCircuit::FiniteCircuit(circuit) => write!(f, "{circuit}"),
            ExtensibleLogicCircuit::Composition(circuit) => write!(f, "{circuit}"),
            ExtensibleLogicCircuit::Iteration(circuit) => write!(f, "{circuit}"),
        }
    }
}
