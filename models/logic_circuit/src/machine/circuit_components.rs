use std::collections::{HashMap, HashSet};
use std::fmt::Display;
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

impl Display for Label {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let string: String = match self {
            Label::Logic(LogicLabel::Not) => "NOT".to_string(),
            Label::Logic(LogicLabel::And) => "AND".to_string(),
            Label::Logic(LogicLabel::Or) => "OR".to_string(),
            Label::Control(ControlLabel::Branch) => "BR".to_string(),
            Label::InOut(InOutLabel::Input) => "IN".to_string(),
            Label::InOut(InOutLabel::Output) => "OUT".to_string(),
        };
        write!(f, "{string}")
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

impl Display for Bool {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Bool::True => write!(f, "T"),
            Bool::False => write!(f, "F"),
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

impl Display for VertexNumbering {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

const LEFT_START: &str = "left-";
const RIGHT_START: &str = "right-";

pub fn left_name_conv_to_name(vertex: &VertexNumbering) -> Option<VertexNumbering> {
    if vertex.to_string().starts_with(LEFT_START) {
        Some(vertex.to_string().split_at(LEFT_START.len()).1.into())
    } else {
        None
    }
}

pub fn name_to_left_name(vertex: &VertexNumbering) -> VertexNumbering {
    format!("{LEFT_START}{}", vertex).into()
}

pub fn right_name_conv_to_name(vertex: &VertexNumbering) -> Option<VertexNumbering> {
    if vertex.to_string().starts_with(RIGHT_START) {
        Some(vertex.to_string().split_at(RIGHT_START.len()).1.into())
    } else {
        None
    }
}

pub fn name_to_right_name(vertex: &VertexNumbering) -> VertexNumbering {
    format!("{RIGHT_START}{}", vertex).into()
}

pub fn iter_name_conv_to_name(v: &VertexNumbering) -> Option<(Number, VertexNumbering)> {
    let str = v.to_string();
    let v: Vec<_> = str.split('-').collect();
    if v.len() != 2 {
        return None;
    }
    let num: Number = v[0].parse::<usize>().ok()?.into();
    let vertex: VertexNumbering = v[1].into();
    Some((num, vertex))
}

pub fn name_to_iter_name(n: Number, v: &VertexNumbering) -> VertexNumbering {
    format!("{}-{}", n.to_string(), v).into()
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
    pub fn appered(&self) -> HashSet<VertexNumbering> {
        self.state.keys().cloned().collect()
    }
    pub fn get_index(&self, index: &VertexNumbering) -> Bool {
        if let Some(bool) = self.state.get(index) {
            bool.clone()
        } else {
            Bool::False
        }
    }
    pub fn set_index(&mut self, index: VertexNumbering, bool: Bool) {
        self.state.insert(index, bool);
    }
    pub fn update_with_input_state(&mut self, input: InputState) -> Option<()> {
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
    pub fn appered_as_true(&self) -> HashSet<VertexNumbering> {
        self.0.iter().filter_map(|(v, b)|{
            if *b == Bool::True {
                Some(v.clone())
            } else {
                None
            }
        }).collect()
    }
    pub fn get_index(&self, index: &VertexNumbering) -> Bool {
        if let Some(bool) = self.0.get(index) {
            bool.clone()
        } else {
            Bool::False
        }
    }
    pub fn get_rid(self, edge_assign: EdgeAssign) -> InputState {
        self.0.into_iter().filter_map(|(v, b)|{
            if edge_assign.contains_as_into(&v) {
                Some((v, b))
            } else {
                None
            }
        }).into()
    }
    pub fn extend(&mut self, other: InputState) {
        self.0.extend(other.0);
    }
    pub fn iterate(self) -> HashMap<VertexNumbering, Bool> {
        self.0
    }
    pub fn retrieve_left(&self) -> InputState {
        self.0.iter().filter_map(|(v, b)|{
            left_name_conv_to_name(v).map(|v| (v, b.clone()))
        }).into()
    }
    pub fn retrieve_right(&self) -> InputState {
        self.0.iter().filter_map(|(v, b)|{
            right_name_conv_to_name(v).map(|v| (v, b.clone()))
        }).into()
    }
    pub fn retrieve_iter(&self, n: Number) -> InputState {
        self.0.iter().filter_map(|(v,b)|{
            iter_name_conv_to_name(v).and_then(|(num, v)|{
                if num == n {
                    Some((v, b.clone()))
                } else {
                    None
                }
            })
        }).into()
    }
    pub fn retrieve_iter_vec(&self) -> Vec<InputState> {
        let mut map: HashMap<Number, HashSet<(VertexNumbering, Bool)>> = HashMap::new();
        let mut max_app = 0;
        self.0.iter().for_each(|(v, b)|{
            if let Some((n, v)) = iter_name_conv_to_name(v) {
                max_app = std::cmp::max(max_app, n.0);
                if let Some(set) = map.get_mut(&n) {
                    set.insert((v, b.clone()));
                } else {
                    let has: HashSet<(VertexNumbering, Bool)> = vec![(v,b.clone())].into_iter().collect();
                    map.insert(n, has);
                }
            }
        });
        (0..=max_app).map(|i|{
            if let Some(set) = map.remove(&i.into()) {
                set.into()
            } else {
                InputState(HashMap::new())
            }
        }).collect()
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
    pub fn appered_as_true(&self) -> HashSet<VertexNumbering> {
        self.0.iter().filter_map(|(v, b)|{
            if *b == Bool::True {
                Some(v.clone())
            } else {
                None
            }
        }).collect()
    }
    pub fn appered(&self) -> HashSet<VertexNumbering> {
        self.0.keys().cloned().collect()
    }
    pub fn get_index(&self, index: &VertexNumbering) -> Bool {
        if let Some(bool) = self.0.get(index) {
            bool.clone()
        } else {
            Bool::False
        }
    }
    pub fn iterate(self) -> HashMap<VertexNumbering, Bool> {
        self.0
    }
}

impl<T> From<T> for OutputState
where
    T: IntoIterator<Item = (VertexNumbering, Bool)>,
{
    fn from(value: T) -> Self {
        OutputState(value.into_iter().collect())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Edge {
    pub from: VertexNumbering,
    pub into: VertexNumbering,
}

impl From<(VertexNumbering, VertexNumbering)> for Edge {
    fn from(value: (VertexNumbering, VertexNumbering)) -> Self {
        Self { from: value.0, into: value.1 }
    }
}

// struct to concat different circuit
// any vertex should appered once
#[derive(Debug, Clone)]
pub struct EdgeAssign(HashSet<Edge>);

impl EdgeAssign {
    pub fn new<T>(value: T) -> Option<Self>
    where
        T: IntoIterator<Item = (VertexNumbering, VertexNumbering)>,
    {
        let mut appeared = HashSet::new();
        let mut map = HashSet::new();
        for (v1, v2) in value {
            if appeared.contains(&v1) {
                return None;
            }
            appeared.insert(v1.clone());
            if appeared.contains(&v2) {
                return None;
            }
            appeared.insert(v2.clone());
            map.insert(Edge {
                from: v1,
                into: v2,
            });
        }
        Some(EdgeAssign(map))
    }
    pub fn get_out_from_in(&self, v: VertexNumbering) -> Option<VertexNumbering> {
        self.0
            .iter()
            .find_map(|Edge{ from, into }| 
                if *into == v { Some(from.clone()) } else { None }
            )
    }
    pub fn iterate(&self) -> impl Iterator<Item = &Edge> {
        self.0.iter()
    }
    pub fn iterate_over_v(&self) -> impl Iterator<Item = (&VertexNumbering, &VertexNumbering)> {
        self.0.iter().map(|Edge { from, into }|(from, into))
    }
    pub fn from_index_to_into_index(&self, from_index: &VertexNumbering) -> Option<&VertexNumbering> {
        self.0.iter().find_map(|Edge { from, into }|{
            if *from == *from_index {
                Some(into)
            } else {
                None
            }
        })
    }
    pub fn contains_as_from(&self, index: &VertexNumbering) -> bool {
        self.0.iter().any(|Edge { from, into: _ }| *from == *index)
    }
    pub fn contains_as_into(&self, index: &VertexNumbering) -> bool {
        self.0.iter().any(|Edge { from: _, into }| *into == *index)
    }
}

pub fn output_to_input_with_edge_assign(output_state: OutputState, edge_assign: EdgeAssign) -> InputState {
    output_state.iterate().into_iter().filter_map(|(from_index, b)|
        edge_assign.from_index_to_into_index(&from_index).map(|into_index|(into_index.clone(), b))
    ).into()
}

pub fn indent(str: String) -> String {
    str.lines().map(|str|{
        format!("    {str}\n")
    }).collect()
}
