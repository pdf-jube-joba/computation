use std::collections::{HashMap, HashSet, VecDeque};
use std::fmt::Display;
use yew::Properties;

// テープの動く方向を表す。
#[derive(Debug, Clone, PartialEq)]
pub enum Direction {
    Right,
    Constant,
    Left,
}

impl TryFrom<&str> for Direction {
    type Error = String;
    fn try_from(value: &str) -> Result<Self, Self::Error> {
        let value = value.trim();
        match value {
            "R" => Ok(Direction::Right),
            "L" => Ok(Direction::Left),
            "C" => Ok(Direction::Constant),
            _ => Err("direction: fail".to_string()),
        }
    }
}

// テープで扱う記号の定義
// 空白記号（スペース）と','と制御記号の含まれない文字列を記号として扱う
// 空の文字列で記号としての空白記号を表す
#[derive(Debug, Default, Clone, PartialEq, Hash, Eq)]
pub struct Sign(String);

impl Sign {
    pub fn blank() -> Sign {
        Sign::try_from("").unwrap()
    }
}

impl Display for Sign {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

// 「両端以外に空白を含むか、 "," を含む文字列」以外は記号として扱う。
// ただし、両端の空白は無視するものとする。
impl TryFrom<&str> for Sign {
    type Error = String;
    fn try_from(value: &str) -> Result<Self, Self::Error> {
        let value = value.trim();
        if value.contains(|char: char| char.is_whitespace() || char == ',') {
            return Err(format!("error on parse sign: input...\"{value}\""));
        }
        Ok(Sign(value.to_string()))
    }
}

pub fn parse_str_to_signs<T>(str: &str) -> Result<T, String>
where
    T: std::iter::FromIterator<Sign>,
{
    str.split_whitespace().map(Sign::try_from).collect()
}

// 左右無限のテープ
// ヘッド部分の読み書きと左右への移動のみが許される
// これの中身を読みたい場合はコストを払って、TapeAsVec を使う
#[derive(Debug, Default, Clone, PartialEq, Properties, Hash, Eq)]
pub struct Tape {
    left: VecDeque<Sign>,
    head: Sign,
    right: VecDeque<Sign>,
}

// テープを簡単に見たり作ったりするための構造体
#[derive(Debug, Default, Clone, PartialEq)]
pub struct TapeAsVec {
    pub left: Vec<Sign>,
    pub head: Sign,
    pub right: Vec<Sign>,
}

impl TapeAsVec {
    pub fn new(
        left: impl IntoIterator<Item = Sign>,
        head: Sign,
        right: impl IntoIterator<Item = Sign>,
    ) -> Self {
        Self {
            left: left.into_iter().collect(),
            head,
            right: right.into_iter().collect(),
        }
    }
    pub fn eq(&self, tape2: &TapeAsVec) -> bool {
        fn same_except_last_blanks(vec1: &[Sign], vec2: &[Sign]) -> bool {
            let iter1 = vec1
                .into_iter()
                .rev()
                .skip_while(|sign| **sign == Sign::blank());
            let iter2 = vec2
                .into_iter()
                .rev()
                .skip_while(|sign| **sign == Sign::blank());
            iter1.eq(iter2)
        }
        let TapeAsVec {
            left: left1,
            head: head1,
            right: right1,
        } = self;
        let TapeAsVec {
            left: left2,
            head: head2,
            right: right2,
        } = tape2;
        same_except_last_blanks(&left1, &left2)
            && head1 == head2
            && same_except_last_blanks(&right1, &right2)
    }
}

impl TryFrom<(Vec<&str>, usize)> for TapeAsVec {
    type Error = String;
    fn try_from(value: (Vec<&str>, usize)) -> Result<Self, Self::Error> {
        let signs: Vec<Sign> = value
            .0
            .into_iter()
            .map(|str| Sign::try_from(str))
            .collect::<Result<_, _>>()?;
        Ok(TapeAsVec {
            left: {
                let mut v = signs[..value.1].to_owned();
                v.reverse();
                v
            },
            head: signs[value.1].to_owned(),
            right: signs[value.1 + 1..].to_owned(),
        })
    }
}

impl From<TapeAsVec> for Tape {
    fn from(TapeAsVec { left, head, right }: TapeAsVec) -> Self {
        Tape {
            left: left.into(),
            head,
            right: right.into(),
        }
    }
}

impl TryFrom<&str> for TapeAsVec {
    type Error = String;
    fn try_from(value: &str) -> Result<Self, Self::Error> {
        let v: Vec<&str> = value.lines().collect();
        if v.len() < 3 {
            return Err("tape: argument is too few".to_owned());
        }
        let left: Vec<Sign> = parse_str_to_signs(v[0])?;
        let head: Sign = v[1].try_into()?;
        let right: Vec<Sign> = parse_str_to_signs(v[2])?;
        Ok(Self { left, head, right })
    }
}

impl TryFrom<String> for TapeAsVec {
    type Error = String;
    fn try_from(value: String) -> Result<Self, Self::Error> {
        let v: Vec<&str> = value.lines().collect();
        if v.len() < 3 {
            return Err("tape: argument is too few".to_owned());
        }
        let left: Vec<Sign> = parse_str_to_signs(v[0])?;
        let head: Sign = v[1].try_into()?;
        let right: Vec<Sign> = parse_str_to_signs(v[2])?;
        Ok(Self { left, head, right })
    }
}

impl From<TapeAsVec> for String {
    fn from(value: TapeAsVec) -> Self {
        let f = |v: Vec<Sign>| {
            v.iter()
                .map(|Sign(sign)| sign.to_owned())
                .collect::<String>()
        };
        format!(
            "l: {} \nh: {}\n r: {}",
            f(value.left),
            value.head,
            f(value.right)
        )
    }
}

impl Display for TapeAsVec {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let TapeAsVec { left, head, right } = &self;
        let mut str = String::new();
        left.iter().rev().for_each(|sign| {
            if *sign == Sign::blank() {
                str.push_str(" ");
            } else {
                str.push_str(&format!("{sign}"));
            }
        });
        str.push_str(&format!("[{}]", head.to_string()));
        right.iter().for_each(|sign| {
            if *sign == Sign::blank() {
                str.push_str(" ");
            } else {
                str.push_str(&format!("{sign}"));
            }
        });
        write!(f, "{}", str)
    }
}

impl Tape {
    pub fn new(
        left: impl IntoIterator<Item = Sign>,
        head: Sign,
        right: impl IntoIterator<Item = Sign>,
    ) -> Self {
        Self {
            left: left.into_iter().collect(),
            head,
            right: right.into_iter().collect(),
        }
    }
    pub fn head_read(&self) -> &Sign {
        &self.head
    }
    fn head_write(&mut self, sign: &Sign) {
        self.head = sign.clone();
    }
    fn move_to(&mut self, m: &Direction) {
        match m {
            Direction::Left => {
                let next_head = self.left.pop_front().unwrap_or_default();
                let old_head = std::mem::replace(&mut self.head, next_head);
                self.right.push_front(old_head);
            }
            Direction::Right => {
                let next_head = self.right.pop_front().unwrap_or_default();
                let old_head = std::mem::replace(&mut self.head, next_head);
                self.left.push_front(old_head);
            }
            Direction::Constant => {}
        }
    }
    fn show(&self) -> TapeAsVec {
        TapeAsVec {
            left: self.left.iter().cloned().collect(),
            head: self.head.clone(),
            right: self.right.iter().cloned().collect(),
        }
    }
}

impl Display for Tape {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let left: String = self.left.iter().map(|sign| format!("{sign} ")).collect();
        writeln!(f, "l:{left}")?;
        writeln!(f, "h:{}", self.head)?;
        let right: String = self.right.iter().map(|sign| format!("{sign} ")).collect();
        writeln!(f, "r:{right}")
    }
}

// マシンの持つ状態の定義
// テープの記号と同じ
#[derive(Debug, Clone, PartialEq, Hash, Eq)]
pub struct State(String);
impl Display for State {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}
impl TryFrom<&str> for State {
    type Error = String;
    fn try_from(value: &str) -> Result<Self, Self::Error> {
        let value = value.trim();
        if value.contains(|char: char| char.is_whitespace() || char == ',') {
            return Err(format!("whitespace contained input:{value}"));
        }
        Ok(State(value.to_string()))
    }
}

#[derive(Debug, Clone, PartialEq, Hash, Eq)]
pub struct CodeKey(Sign, State);
#[derive(Debug, Clone, PartialEq)]
pub struct CodeValue(Sign, State, Direction);

#[derive(Debug, Clone, PartialEq)]
pub struct CodeEntry(CodeKey, CodeValue);

impl CodeEntry {
    pub fn key_sign(&self) -> Sign {
        self.0 .0.clone()
    }
    pub fn key_state(&self) -> State {
        self.0 .1.clone()
    }
    pub fn value_sign(&self) -> Sign {
        self.1 .0.clone()
    }
    pub fn value_state(&self) -> State {
        self.1 .1.clone()
    }
    pub fn value_direction(&self) -> Direction {
        self.1 .2.clone()
    }
    pub fn from_tuple(
        key_sign: Sign,
        key_state: State,
        value_sign: Sign,
        value_state: State,
        value_direction: Direction,
    ) -> Self {
        CodeEntry(
            CodeKey(key_sign, key_state),
            CodeValue(value_sign, value_state, value_direction),
        )
    }
}

impl TryFrom<&str> for CodeEntry {
    type Error = String;
    fn try_from(value: &str) -> Result<Self, Self::Error> {
        if value.contains(|char: char| char.is_control()) {
            return Err("code-entry: some control char contained".to_string());
        };
        let v: Vec<&str> = value.split(',').collect();
        if v.len() < 5 {
            return Err("code-entry: argument is too few".to_string());
        }
        let code_key: CodeKey = CodeKey(v[0].try_into()?, v[1].try_into()?);
        let code_value: CodeValue = CodeValue(v[2].try_into()?, v[3].try_into()?, v[4].try_into()?);
        Ok(CodeEntry(code_key, code_value))
    }
}

#[derive(Debug, Default, Clone, PartialEq, Properties)]
pub struct Code {
    hash: HashMap<CodeKey, CodeValue>,
}

impl Code {
    fn code(&self) -> &HashMap<CodeKey, CodeValue> {
        &self.hash
    }
    pub fn code_as_vec(&self) -> Vec<CodeEntry> {
        self.hash
            .iter()
            .map(|(k, v)| CodeEntry(k.clone(), v.clone()))
            .collect()
    }
    pub fn add(&mut self, CodeEntry(k, v): CodeEntry) {
        self.hash.insert(k, v);
    }
    pub fn from_iter_entry(iter: impl IntoIterator<Item = CodeEntry>) -> Self {
        Code {
            hash: HashMap::from_iter(iter.into_iter().map(|CodeEntry(k, v)| (k, v))),
        }
    }
}

impl TryFrom<&str> for Code {
    type Error = String;
    fn try_from(value: &str) -> Result<Self, Self::Error> {
        let mut code = Code::default();
        for (index, str) in value.lines().enumerate() {
            match CodeEntry::try_from(str) {
                Ok(entry) => {
                    code.add(entry);
                }
                Err(err) => {
                    return Err(format!("{} at line {}", err, index));
                }
            }
        }
        Ok(code)
    }
}

// Turing machine は次のものから構成されている。
// Σ:有限集合...テープに用いる記号
// b:Σ...空白記号
// Q:有限集合...マシンの状態
// q_init:Q...マシンの初期状態
// F:subset of Q...マシンの受理状態全体
// δ:(Q,Σ\F) -> (Q,Σ,{E,R,C})...マシンの遷移関数
// ただし、実装上は次のように固定してしまう
// ΣやQはある無限集合（可能なマシンの用いうる記号や状態の集合）Sign, State の部分集合を（暗黙的に）指しているものとし、
// δを（有限な）Vec<(Sign, State), (Sign, State, {L,R,C})> により実装することで、
// このHashMapに存在するSignやStateが「実は考えていたQやΣである」とする。
// また、マシンの停止は以下の二つの可能性があるものとする。
// - マシンの状態が accepted_state に含まれる。
// - 部分関数である遷移関数の定義域に含まれない。
#[derive(Debug, Clone, PartialEq)]
pub struct TuringMachine {
    init_state: State,
    accepted_state: Vec<State>,
    code: Vec<CodeEntry>,
}

impl TuringMachine {
    pub fn new(
        init_state: State,
        accepted_state: impl IntoIterator<Item = State>,
        code: impl IntoIterator<Item = CodeEntry>,
    ) -> Result<Self, ()> {
        let accepted_state: Vec<State> = accepted_state.into_iter().collect();
        let code: Vec<CodeEntry> = code
            .into_iter()
            .map(|entry| {
                if accepted_state.contains(&entry.key_state()) {
                    Err(())
                } else {
                    Ok(entry)
                }
            })
            .collect::<Result<_, _>>()?;
        Ok(TuringMachine {
            init_state,
            accepted_state,
            code,
        })
    }
    pub fn init_state(&self) -> &State {
        &self.init_state
    }
    pub fn accepted_state(&self) -> &Vec<State> {
        &self.accepted_state
    }
    pub fn code(&self) -> &Vec<CodeEntry> {
        &self.code
    }
    // return possible sign
    pub fn signs(&self) -> Vec<Sign> {
        self.code
            .iter()
            .flat_map(|CodeEntry(CodeKey(sign1, _), CodeValue(sign2, _, _))| {
                vec![sign1.clone(), sign2.clone()]
            })
            .collect()
    }
    pub fn states(&self) -> Vec<State> {
        let mut state: Vec<State> = vec![self.init_state.clone()];
        state.extend_from_slice(&self.accepted_state);
        state.extend(self.code.iter().flat_map(
            |CodeEntry(CodeKey(_, state1), CodeValue(_, state2, _))| {
                vec![state1.clone(), state2.clone()]
            },
        ));
        state
    }
}

// TuringMachine の計算過程を表す。
#[derive(Debug, Clone, PartialEq)]
pub struct TuringMachineState {
    state: State,
    tape: Tape,
}

impl TuringMachineState {
    pub fn new(state: State, tape: Tape) -> Self {
        TuringMachineState { state, tape }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TuringMachineSet {
    machine_code: Code,
    accepted_state: HashSet<State>,
    machine_state: TuringMachineState,
    made_by: TuringMachine,
}

impl TuringMachineSet {
    pub fn new(machine: TuringMachine, tape: TapeAsVec) -> Self {
        let TuringMachine {
            init_state,
            accepted_state,
            code,
        } = machine.clone();
        let machine_code = Code::from_iter_entry(code);
        let accepted_state = HashSet::from_iter(accepted_state);
        let machine_state =
            TuringMachineState::new(init_state, Tape::new(tape.left, tape.head, tape.right));
        TuringMachineSet {
            machine_code,
            accepted_state,
            machine_state,
            made_by: machine,
        }
    }
    fn now_key(&self) -> CodeKey {
        CodeKey(
            self.machine_state.tape.head_read().clone(),
            self.machine_state.state.clone(),
        )
    }
    pub fn now_state(&self) -> &State {
        &self.machine_state.state
    }
    pub fn now_tape(&self) -> TapeAsVec {
        self.machine_state.tape.show()
    }
    pub fn code_as_vec(&self) -> &Vec<CodeEntry> {
        &self.made_by.code
    }
    pub fn is_terminate(&self) -> bool {
        self.accepted_state.contains(&self.machine_state.state)
            || !self.machine_code.code().contains_key(&self.now_key())
    }
    pub fn is_accepted(&self) -> bool {
        self.accepted_state.contains(&self.machine_state.state)
    }
    pub fn next_step(&self) -> Result<CodeEntry, ()> {
        if self.is_terminate() {
            return Err(());
        }
        let key = self.now_key();
        let value = self.machine_code.code().get(&key).unwrap().clone();
        Ok(CodeEntry(key, value))
    }
    fn one_step(&mut self) {
        if !self.is_terminate() {
            let hash = self.machine_code.code();
            let key = self.now_key();
            let CodeValue(sign, state, direction) = hash.get(&key).unwrap();
            self.machine_state.tape.head_write(sign);
            self.machine_state.tape.move_to(direction);
            self.machine_state.state = state.clone();
        }
    }
    pub fn step(&mut self, num: usize) -> Result<(), usize> {
        for i in 0..num {
            if self.is_terminate() {
                return Err(i);
            }
            self.one_step();
        }
        Ok(())
    }
    pub fn result(&self) -> Result<TapeAsVec, String> {
        if !self.is_terminate() {
            return Err("not terminated".to_string());
        }
        Ok(self.now_tape())
    }
}

impl Display for TuringMachineSet {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "code:")?;
        for (CodeKey(k1, k2), CodeValue(v1, v2, v3)) in self.machine_code.hash.iter() {
            writeln!(f, "{k1}, {k2}, {v1}, {v2}, {v3:?}")?;
        }
        writeln!(f, "state: {}", self.machine_state.state)?;
        writeln!(f, "tape: {:?}", self.machine_state.tape)
    }
}
