use crate::{assign, lv, rv};
use std::rc::Rc;
use turing_machine::machine::{Sign, Tape};
use utils::{Machine, RenderBlock, StepResult, TextCodec, parse::ParseTextCodec};

use crate::rec_tm_ir::{Block, Function, Program, Stmt};

use super::RecTmIrMachine;

fn run_until_halt(mut machine: RecTmIrMachine, limit: usize) -> Result<Tape, String> {
    for _ in 0..limit {
        match machine.step(())? {
            StepResult::Continue { next, .. } => {
                machine = next;
            }
            StepResult::Halt { output } => {
                return Ok(output);
            }
        }
    }
    Err("step limit exceeded".to_string())
}

fn tape_from_machine(machine: &RecTmIrMachine) -> Result<Tape, String> {
    let value = RecTmIrMachine::render(machine.snapshot());
    let tape = value
        .last()
        .and_then(|v| serde_json::to_value(v).ok())
        .ok_or_else(|| "snapshot tape container missing".to_string())?;
    let children = tape
        .get("children")
        .and_then(|v| v.as_array())
        .ok_or_else(|| "snapshot tape children missing".to_string())?;
    let mut signs = Vec::new();
    let mut head = None;
    for (idx, child) in children.iter().enumerate() {
        let obj = child
            .as_object()
            .ok_or_else(|| "tape child is not an object".to_string())?;
        let text = obj
            .get("text")
            .and_then(|v| v.as_str())
            .ok_or_else(|| "tape cell missing text".to_string())?;
        signs.push(<Sign as TextCodec>::parse(text)?);
        if obj.get("className").and_then(|v| v.as_str()) == Some("highlight") {
            head = Some(idx);
        }
    }
    Tape::from_vec(signs, head.unwrap_or(0))
}

fn head_text(tape: Tape) -> Result<String, String> {
    let (signs, head) = tape.into_vec();
    signs
        .get(head)
        .map(Sign::print)
        .ok_or_else(|| "highlighted tape cell not found".to_string())
}

fn ms(s: &str) -> Sign {
    TextCodec::parse(s).unwrap()
}

#[test]
fn tape_left_right() {
    let program = Program {
        alphabet: vec![ms("a")],
        functions: vec![Rc::new(Function {
            name: "main".to_string(),
            blocks: vec![Block {
                label: "entry".to_string(),
                body: vec![
                    assign!(lv!(@), rv!(const ms("a"))),
                    Stmt::Rt,
                    Stmt::Rt,
                    assign!(lv!(@), rv!(const ms("a"))),
                    Stmt::Lt,
                    Stmt::Lt,
                    Stmt::Lt,
                    Stmt::Lt,
                    assign!(lv!(@), rv!(const ms("a"))),
                ],
            }],
        })],
    };
    let tape = <RecTmIrMachine as Machine>::parse_ainput("|-|").unwrap();
    let mut machine = RecTmIrMachine::make(program, tape).unwrap();
    let mut halted_tape = None;
    let mut current_tape = tape_from_machine(&machine).unwrap();

    for _ in 0..10 {
        let snapshot = current_tape.clone();
        eprintln!("{}", snapshot.print());
        match machine.step(()) {
            Ok(StepResult::Continue { next, .. }) => {
                current_tape = tape_from_machine(&next).unwrap();
                machine = next;
            }
            Ok(StepResult::Halt { output }) => {
                halted_tape = Some(output);
                break;
            }
            Err(e) => panic!("step failed: {e}"),
        }
    }

    let snapshot = halted_tape.unwrap_or(current_tape);
    eprintln!("{}", snapshot.print());

    let expected_tape: crate::rec_tm_ir::Tape = "|a|-,a,-,a".parse_tc().unwrap();

    assert!(snapshot.eq(&expected_tape))
}

#[test]
fn call_does_not_share_env() {
    let code = r#"
alphabet: (a, b)
fn f() {
  label entry: {
    x := @
  }
}

fn main() {
  label entry: {
    x := @
    RT
    call f
    LT
    @ := x
  }
}
"#;
    let program = <RecTmIrMachine as Machine>::parse_code(code).unwrap();
    let tape = <RecTmIrMachine as Machine>::parse_ainput("-|a|b").unwrap();
    let machine = RecTmIrMachine::make(program, tape).unwrap();
    let snapshot = run_until_halt(machine, 64).unwrap();

    let head = head_text(snapshot).unwrap();
    assert_eq!(head, "a");
}

#[test]
fn recursion_is_rejected() {
    let code = r#"
alphabet: (a)
fn f() {
  label entry: {
    call f
  }
}

fn main() {
  label entry: {
    call f
  }
}
"#;
    let program = <RecTmIrMachine as Machine>::parse_code(code).unwrap();
    let tape = <RecTmIrMachine as Machine>::parse_ainput("-|a|-").unwrap();
    let result = RecTmIrMachine::make(program, tape);
    assert!(result.is_err());
}

#[test]
fn scan_and_mark_tape() {
    let code = r#"
alphabet: (m, a, b, x)
fn main() {
  label entry: {
    mark := @
    jump scan
  }
  label scan: {
    RT
    cur := @
    jump done if cur == const x
    jump scan
  }
  label done: {
    @ := mark
  }
}
"#;
    let program = <RecTmIrMachine as Machine>::parse_code(code).unwrap();
    let tape = <RecTmIrMachine as Machine>::parse_ainput("-|m|a,b,x,-").unwrap();
    let machine = RecTmIrMachine::make(program, tape).unwrap();
    let snapshot = run_until_halt(machine, 64).unwrap();
    let head = head_text(snapshot).unwrap();
    assert_eq!(head, "m");
}

#[test]
fn call_chain_does_not_share_env() {
    let code = r#"
alphabet: (a, b)
fn g() {
  label entry: {
    RT
    p := @
    LT
  }
}

fn f() {
  label entry: {
    call g
    @ := p
  }
}

fn main() {
  label entry: {
    x := @
    call f
  }
}
"#;
    let program = <RecTmIrMachine as Machine>::parse_code(code).unwrap();
    let tape = <RecTmIrMachine as Machine>::parse_ainput("-|a|b").unwrap();
    let machine = RecTmIrMachine::make(program, tape).unwrap();
    let snapshot = run_until_halt(machine, 64).unwrap();
    let head = head_text(snapshot).unwrap();
    assert_eq!(head, "-");
}

#[test]
fn nested_loop_breaks_resolve() {
    let code = r#"
alphabet: (a, b, c)
fn main() {
  label entry: {
    jump scan_b
  }
  label scan_b: {
    RT
    v := @
    jump scan_c if v == const b
    jump scan_b
  }
  label scan_c: {
    RT
    v := @
    jump done if v == const c
    jump scan_b
  }
  label done: {
  }
}
"#;
    let program = <RecTmIrMachine as Machine>::parse_code(code).unwrap();
    let tape = <RecTmIrMachine as Machine>::parse_ainput("-|a|b,c").unwrap();
    let machine = RecTmIrMachine::make(program, tape).unwrap();
    let snapshot = run_until_halt(machine, 64).unwrap();
    let head = head_text(snapshot).unwrap();
    assert_eq!(head, "c");
}

#[test]
fn repeated_calls_keep_env_isolated() {
    let code = r#"
alphabet: (a, b, c)
fn writer() {
  label entry: {
    jump second if flag == const a
    flag := const a
    @ := const a
    return
  }
  label second: {
    @ := const b
    return
  }
}
fn main() {
  label entry: {
    call writer
    call writer
  }
}
"#;
    let program = <RecTmIrMachine as Machine>::parse_code(code).unwrap();
    let tape = <RecTmIrMachine as Machine>::parse_ainput("|-|").unwrap();
    let machine = RecTmIrMachine::make(program, tape).unwrap();
    let snapshot = run_until_halt(machine, 16).unwrap();
    let head = head_text(snapshot).unwrap();
    assert_eq!(head, "a");
}

#[test]
fn break_jumps_to_next_block() {
    let code = r#"
alphabet: (a, b)
fn main() {
  label first: {
    @ := const a
    break
    @ := const b
  }
  label second: {
    RT
    @ := const b
  }
}
"#;
    let program = <RecTmIrMachine as Machine>::parse_code(code).unwrap();
    let tape = <RecTmIrMachine as Machine>::parse_ainput("|-|").unwrap();
    let machine = RecTmIrMachine::make(program, tape).unwrap();
    let snapshot = run_until_halt(machine, 16).unwrap();
    let head = head_text(snapshot).unwrap();
    assert_eq!(head, "b");
}

#[test]
fn continue_loops_current_block() {
    let code = r#"
alphabet: (a)
fn main() {
  label entry: {
    @ := const a
    continue
    @ := const -
  }
  label next: {
    @ := const -
  }
}
"#;
    let program = <RecTmIrMachine as Machine>::parse_code(code).unwrap();
    let tape = <RecTmIrMachine as Machine>::parse_ainput("|-|").unwrap();
    let machine = RecTmIrMachine::make(program, tape).unwrap();
    let result = run_until_halt(machine, 8);
    assert!(result.is_err());
}
