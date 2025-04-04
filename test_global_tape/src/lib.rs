use std::sync::{LazyLock, Mutex};

use wasm_bindgen::prelude::*;

pub fn alert(message: &str) {
    web_sys::window()
        .unwrap()
        .alert_with_message(message)
        .unwrap();
}

mod tape {
    #[derive(Debug, Clone)]
    pub struct Tape {
        left: Vec<String>,
        head: String,
        right: Vec<String>,
    }

    // struct は Tape だが、Tape として的確かどうかをテストする。
    pub fn tape_parse(left: &str, head: &str, right: &str) -> Tape {
        let left: Vec<_> = left.split(',').map(|s| s.trim().to_owned()).collect();
        let head = head.trim().to_owned();
        let right: Vec<_> = right
            .split(',')
            .map(|s| s.trim().to_owned())
            .rev()
            .collect();
        Tape { left, head, right }
    }

    pub fn move_right(tape: &mut Tape) {
        let new_head = tape.right.pop().unwrap_or_default();
        let old_head = std::mem::replace(&mut tape.head, new_head);
        tape.left.push(old_head);
    }

    pub fn move_left(tape: &mut Tape) {
        let new_head = tape.left.pop().unwrap_or_default();
        let old_head = std::mem::replace(&mut tape.head, new_head);
        tape.right.push(old_head);
    }

    pub fn head(tape: &Tape) -> String {
        tape.head.clone()
    }

    pub fn right(tape: &Tape) -> Vec<String> {
        tape.right.clone()
    }

    pub fn left(tape: &Tape) -> Vec<String> {
        tape.left.clone()
    }
}

// many global mutable tapes
static TAPES: LazyLock<Mutex<Vec<tape::Tape>>> = LazyLock::new(|| Mutex::new(vec![]));

#[wasm_bindgen]
pub fn new_tape(left: String, head: String, right: String) -> usize {
    let mut tapes = TAPES.lock().unwrap();
    let l = tapes.len();
    tapes.push(tape::tape_parse(&left, &head, &right));
    l
}

#[wasm_bindgen]
pub fn mutate_tape(id: usize, left: String, head: String, right: String) {
    let mut tapes = TAPES.lock().unwrap();
    if let Some(tape) = tapes.get_mut(id) {
        *tape = tape::tape_parse(&left, &head, &right);
    } else {
        alert("Invalid tape ID");
    }
}

#[wasm_bindgen]
pub fn head(tape_id: usize) -> String {
    let tapes = TAPES.lock().unwrap();
    if let Some(tape) = tapes.get(tape_id) {
        tape::head(tape)
    } else {
        alert("Invalid tape ID");
        String::new()
    }
}

#[wasm_bindgen]
pub fn left(tape_id: usize) -> Vec<String> {
    let tapes = TAPES.lock().unwrap();
    if let Some(tape) = tapes.get(tape_id) {
        tape::left(tape)
    } else {
        alert("Invalid tape ID");
        vec![]
    }
}

#[wasm_bindgen]
pub fn right(tape_id: usize) -> Vec<String> {
    let tapes = TAPES.lock().unwrap();
    if let Some(tape) = tapes.get(tape_id) {
        tape::right(tape)
    } else {
        alert("Invalid tape ID");
        vec![]
    }
}

#[wasm_bindgen]
pub fn move_right(tape_id: usize) {
    let mut tapes = TAPES.lock().unwrap();
    if let Some(tape) = tapes.get_mut(tape_id) {
        tape::move_right(tape);
    } else {
        alert("Invalid tape ID");
    }
}

#[wasm_bindgen]
pub fn move_left(tape_id: usize) {
    let mut tapes = TAPES.lock().unwrap();
    if let Some(tape) = tapes.get_mut(tape_id) {
        tape::move_left(tape);
    } else {
        alert("Invalid tape ID");
    }
}
