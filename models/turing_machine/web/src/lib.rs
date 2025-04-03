use serde::Serialize;
use turing_machine_core::machine::{Direction, Sign, Tape};
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
#[derive(Debug, Clone, Serialize)]
pub struct TapeWeb {
    left: Vec<String>,
    head: String,
    right: Vec<String>,
}

impl TryFrom<TapeWeb> for Tape {
    type Error = String;
    fn try_from(TapeWeb { left, head, right }: TapeWeb) -> Result<Self, Self::Error> {
        let left: Vec<_> = left
            .into_iter()
            .map(|s| Sign::try_from(s.trim()))
            .collect::<Result<_, _>>()?;
        let right: Vec<_> = right
            .into_iter()
            .rev()
            .map(|s| Sign::try_from(s.as_ref()))
            .collect::<Result<_, _>>()?;
        let head: Sign = Sign::try_from(head.trim())?;
        Ok(Tape::new(left, head, right))
    }
}

impl From<Tape> for TapeWeb {
    fn from(Tape { left, head, right }: Tape) -> Self {
        TapeWeb {
            left: left.into_iter().map(|s| s.to_string()).collect(),
            head: format!("{head}"),
            right: right.into_iter().rev().map(|s| s.to_string()).collect(),
        }
    }
}

// struct は TapeWeb だが、Tape として的確かどうかをテストする。
#[wasm_bindgen]
pub fn tape_parse(left: &str, head: &str, right: &str) -> TapeWeb {
    let left: Vec<_> = left.split(',').map(|s| s.trim().to_owned()).collect();
    let head = head.trim().to_owned();
    let right: Vec<_> = right.split(',').map(|s| s.trim().to_owned()).collect();
    let tapeweb = TapeWeb { left, head, right };
    match Tape::try_from(tapeweb.clone()) {
        Ok(_) => tapeweb,
        Err(err) => TapeWeb {
            left: vec![err],
            head: String::new(),
            right: vec![],
        },
    }
}

#[wasm_bindgen]
pub fn move_right(tape: &mut TapeWeb) {
    let mut tape_as: Tape = tape.clone().try_into().unwrap();
    tape_as.move_to(&Direction::Right);
    *tape = tape_as.into();
}

#[wasm_bindgen]
pub fn move_left(tape: &mut TapeWeb) {
    let mut tape_as: Tape = tape.clone().try_into().unwrap();
    tape_as.move_to(&Direction::Left);
    *tape = tape_as.into();
}

#[wasm_bindgen]
pub fn head(tape: &TapeWeb) -> String {
    tape.head.clone()
}
#[wasm_bindgen]
pub fn right(tape: &TapeWeb) -> Vec<String> {
    tape.right.clone()
}
#[wasm_bindgen]
pub fn left(tape: &TapeWeb) -> Vec<String> {
    tape.left.clone()
}
