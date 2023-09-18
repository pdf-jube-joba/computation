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
    format!("{LEFT_START}{}", vertex.to_string()).into()
}

pub fn right_name_conv_to_name(vertex: &VertexNumbering) -> Option<VertexNumbering> {
    if vertex.to_string().starts_with(RIGHT_START) {
        Some(vertex.to_string().split_at(RIGHT_START.len()).1.into())
    } else {
        None
    }
}

pub fn name_to_right_name(vertex: &VertexNumbering) -> VertexNumbering {
    format!("{RIGHT_START}{}", vertex.to_string()).into()
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
    format!("{}-{}", n.to_string(), v.to_string()).into()
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
    pub fn get_index(&self, index: &VertexNumbering) -> Option<Bool> {
        self.state.get(index).cloned()
    }
    pub fn get_mut_index(&mut self, index: &VertexNumbering) -> Option<&mut Bool> {
        self.state.get_mut(index)
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
    pub fn get_index(&self, index: &VertexNumbering) -> Bool {
        if let Some(bool) = self.0.get(index) {
            bool.clone()
        } else {
            Bool::False
        }
    }
    pub fn extend(&mut self, other: InputState) {
        self.0.extend(other.0);
    }
    pub fn iterate(self) -> HashMap<VertexNumbering, Bool> {
        self.0
    }
    pub fn retrieve_left(&self) -> InputState {
        self.0.iter().filter_map(|(v, b)|{
            left_name_conv_to_name(&v).map(|v| (v, b.clone()))
        }).into()
    }
    pub fn retrieve_right(&self) -> InputState {
        self.0.iter().filter_map(|(v, b)|{
            right_name_conv_to_name(&v).map(|v| (v, b.clone()))
        }).into()
    }
    pub fn retrieve_iter(&self, n: Number) -> InputState {
        self.0.iter().filter_map(|(v,b)|{
            iter_name_conv_to_name(&v).and_then(|(num, v)|{
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
