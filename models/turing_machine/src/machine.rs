use serde::Serialize;
use utils::alphabet::Alphabet; // Import Alphabet from the utils crate

// テープの動く方向を表す。
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub enum Direction {
    Right,
    Constant,
    Left,
}

// テープで扱う記号の定義
// 空白記号（None）と制御記号の含まれない文字列を記号として扱う
// Alphabet は空白ではない
#[derive(Debug, Default, Clone, PartialEq, Hash, Eq, Serialize)]
pub struct Sign(pub(crate) Option<Alphabet>);

impl Sign {
    pub fn blank() -> Sign {
        Sign(None)
    }
}

// 左右無限のテープ
// ヘッド部分の読み書きと左右への移動のみが許される
// テープの左右端には空白記号が無限に並んでいるものとする
// left[0] が左端で right[0] が右端 => テープとしては、 left[0] ... left[n] [head] right[m] ... right[0]
#[derive(Debug, Default, Clone, Serialize)]
pub struct Tape {
    left: Vec<Sign>,
    head: Sign,
    right: Vec<Sign>,
}

impl PartialEq for Tape {
    // 空白記号のみの部分は無視して比較する
    fn eq(&self, other: &Self) -> bool {
        fn same_except_last_blanks(vec1: &[Sign], vec2: &[Sign]) -> bool {
            let max_len = vec1.len().max(vec2.len());
            let blank = Sign::blank();

            let iter1 = vec1
                .iter()
                .rev()
                .chain(std::iter::repeat(&blank))
                .take(max_len);
            let iter2 = vec2
                .iter()
                .rev()
                .chain(std::iter::repeat(&blank))
                .take(max_len);
            iter1.eq(iter2)
        }
        let Tape {
            left: left1,
            head: head1,
            right: right1,
        } = self;
        let Tape {
            left: left2,
            head: head2,
            right: right2,
        } = other;
        same_except_last_blanks(left1, left2)
            && head1 == head2
            && same_except_last_blanks(right1, right2)
    }
}

impl Tape {
    pub fn from_vec(v: impl IntoIterator<Item = Sign>, pos: usize) -> Result<Self, String> {
        let v: Vec<Sign> = v.into_iter().collect();
        if pos > v.len() {
            return Err("Position out of bounds".to_string());
        }
        let left = v[..pos].to_vec();
        let head = v.get(pos).cloned().unwrap_or_default();
        let mut right = v[pos + 1..].to_vec();
        right.reverse();
        Ok(Self { left, head, right })
    }
    pub fn head_read(&self) -> &Sign {
        &self.head
    }
    pub fn head_write(&mut self, sign: &Sign) {
        self.head = sign.clone();
    }
    pub fn move_to(&mut self, m: &Direction) {
        match m {
            Direction::Left => {
                let next_head = self.left.pop().unwrap_or_default();
                let old_head = std::mem::replace(&mut self.head, next_head);
                self.right.push(old_head);
            }
            Direction::Right => {
                let next_head = self.right.pop().unwrap_or_default();
                let old_head = std::mem::replace(&mut self.head, next_head);
                self.left.push(old_head);
            }
            Direction::Constant => {}
        }
    }
    pub fn into_vec(&self) -> (Vec<Sign>, usize) {
        let mut v = self.left.clone();
        let pos = v.len();
        v.push(self.head.clone());
        let mut right = self.right.clone();
        right.reverse();
        v.extend(right);
        (v, pos)
    }
}

// マシンの持つ状態の定義
// テープの記号と同じ
#[derive(Debug, Clone, PartialEq, Hash, Eq, Serialize)]
pub struct State(pub(crate) Alphabet);

pub type CodeEntry = ((Sign, State), (Sign, State, Direction));
pub type Code = Vec<CodeEntry>;

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct TuringMachineDefinition {
    init_state: State,
    accepted_state: Vec<State>,
    code: Code,
}

impl TuringMachineDefinition {
    pub fn new(
        init_state: State,
        accepted_state: impl IntoIterator<Item = State>,
        code: impl IntoIterator<Item = CodeEntry>,
    ) -> Result<Self, anyhow::Error> {
        // Changed from String to anyhow::Error
        let accepted_state: Vec<State> = accepted_state.into_iter().collect();
        let code: Code = code
            .into_iter()
            .map(|entry| {
                if accepted_state.contains(&entry.0.1) {
                    Err(anyhow::anyhow!("Code contains accepted state"))
                } else {
                    Ok(entry)
                }
            })
            .collect::<Result<_, _>>()?;
        Ok(TuringMachineDefinition {
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
    pub fn code(&self) -> &Code {
        &self.code
    }
    pub fn signs(&self) -> Vec<Sign> {
        self.code
            .iter()
            .flat_map(|((sign1, _), (sign2, _, _))| vec![sign1.clone(), sign2.clone()])
            .collect()
    }
    pub fn states(&self) -> Vec<State> {
        let mut state: Vec<State> = vec![self.init_state.clone()];
        state.extend_from_slice(&self.accepted_state);
        state.extend(
            self.code
                .iter()
                .flat_map(|((_, state1), (_, state2, _))| vec![state1.clone(), state2.clone()]),
        );
        state
    }
    pub fn get_now_entry(&self, key: &(Sign, State)) -> Option<(usize, &(Sign, State, Direction))> {
        self.code
            .iter()
            .enumerate()
            .find(|(_, ((sign, state), _))| sign == &key.0 && state == &key.1)
            .map(|(i, (_, next))| (i, next))
    }
    pub fn get_next_state(&self, key: &(Sign, State)) -> Option<&(Sign, State, Direction)> {
        self.code
            .iter()
            .find(|((sign, state), _)| sign == &key.0 && state == &key.1)
            .map(|(_, next)| next)
    }
}

// TuringMachine の計算過程を表す。
#[derive(Debug, Clone, PartialEq, Serialize)]
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
    machine_definition: TuringMachineDefinition,
    machine_state: TuringMachineState,
}

impl TuringMachineSet {
    pub fn new(machine: TuringMachineDefinition, tape: Tape) -> Self {
        let init_state = machine.init_state.clone();
        TuringMachineSet {
            machine_definition: machine,
            machine_state: TuringMachineState::new(init_state, tape),
        }
    }
    fn now_key(&self) -> (Sign, State) {
        (
            self.machine_state.tape.head_read().clone(),
            self.machine_state.state.clone(),
        )
    }
    pub fn now_state(&self) -> &State {
        &self.machine_state.state
    }
    pub fn now_tape(&self) -> &Tape {
        &self.machine_state.tape
    }
    pub fn next_code(&self) -> Option<(usize, &(Sign, State, Direction))> {
        if self.is_terminate() {
            return None;
        }
        self.machine_definition.get_now_entry(&self.now_key())
    }
    pub fn code(&self) -> &Code {
        &self.machine_definition.code
    }
    pub fn init_state(&self) -> &State {
        &self.machine_definition.init_state
    }
    pub fn accepted_state(&self) -> &Vec<State> {
        &self.machine_definition.accepted_state
    }
    pub fn is_accepted(&self) -> bool {
        self.machine_definition
            .accepted_state
            .contains(&self.machine_state.state)
    }
    pub fn is_terminate(&self) -> bool {
        self.is_accepted()
            || self
                .machine_definition
                .get_next_state(&self.now_key())
                .is_none()
    }
    fn one_step(&mut self) {
        if !self.is_terminate() {
            let key = self.now_key();
            let (sign, state, direction) = self.machine_definition.get_next_state(&key).unwrap();
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
    pub fn result(&self) -> Result<Tape, anyhow::Error> {
        // Changed from String to anyhow::Error
        if !self.is_terminate() {
            return Err(anyhow::anyhow!("not terminated"));
        }
        Ok(self.now_tape().clone())
    }
}
