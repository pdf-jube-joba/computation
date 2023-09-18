use std::collections::{HashMap, HashSet};
use utils::number::*;
use super::circuit_components::*;

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
    pub fn iterate_as_set(&self) -> impl Iterator<Item = (&VertexNumbering, &Label, &Option<Bool>)> {
        self.label_and_initial_state.iter().map(|(v, (l, b))| (v, l, b))
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
    ) -> Option<Self> {
        unimplemented!()
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

impl IterationCircuit {
    pub fn iter(&self) -> ExtensibleLogicCircuit {
        self.iter.clone()
    }
    pub fn pre_to_post_edge(&self) -> EdgeAssign {
        self.pre_to_post.clone()
    }
    pub fn post_to_pre(&self) -> EdgeAssign {
        self.post_to_pre.clone()
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
