use utils::{Machine, TextCodec};

use crate::{flatten_program, RecTmIrMachine, Snapshot, Stmt};

fn run_until_halt(machine: &mut RecTmIrMachine, limit: usize) -> EnvironmentResult {
    for _ in 0..limit {
        if let Some(env) = machine.step(())? {
            return Ok(env);
        }
    }
    Err("step limit exceeded".to_string())
}

type EnvironmentResult = Result<crate::Environment, String>;

fn head_text(snapshot: Snapshot) -> Result<String, String> {
    let value: serde_json::Value = snapshot.into();
    let arr = value
        .as_array()
        .ok_or_else(|| "snapshot json is not an array".to_string())?;
    let tape = arr
        .last()
        .and_then(|v| v.as_object())
        .ok_or_else(|| "snapshot tape container missing".to_string())?;
    let children = tape
        .get("children")
        .and_then(|v| v.as_array())
        .ok_or_else(|| "snapshot tape children missing".to_string())?;
    for child in children {
        let obj = child
            .as_object()
            .ok_or_else(|| "tape child is not an object".to_string())?;
        if obj.get("className").and_then(|v| v.as_str()) == Some("highlight") {
            return obj
                .get("text")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string())
                .ok_or_else(|| "highlighted tape cell missing text".to_string());
        }
    }
    Err("highlighted tape cell not found".to_string())
}

#[test]
fn call_does_not_share_env() {
    let code = r#"
fn f(x) {
  READ x
}

fn main() {
  READ x
  RT
  call f(x)
  LT
  STOR x
}
"#;
    let program = <RecTmIrMachine as Machine>::parse_code(code).unwrap();
    let tape = <RecTmIrMachine as Machine>::parse_ainput("-|a|b").unwrap();
    let mut machine = RecTmIrMachine::make(program, tape).unwrap();

    let output = run_until_halt(&mut machine, 64).unwrap();
    assert_eq!(output.print().trim(), "x = a");

    let head = head_text(machine.current()).unwrap();
    assert_eq!(head, "a");
}

#[test]
fn recursion_is_rejected() {
    let code = r#"
fn f(x) {
  call f(x)
}

fn main() {
  call f(x)
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
fn main() {
  READ mark
  loop L: {
    RT
    READ cur
    if cur == x break L
  }
  STOR mark
}
"#;
    let program = <RecTmIrMachine as Machine>::parse_code(code).unwrap();
    let tape = <RecTmIrMachine as Machine>::parse_ainput("-|m|a,b,x,-").unwrap();
    let mut machine = RecTmIrMachine::make(program, tape).unwrap();

    let _ = run_until_halt(&mut machine, 64).unwrap();
    let head = head_text(machine.current()).unwrap();
    assert_eq!(head, "m");
}

#[test]
fn call_chain_does_not_share_env() {
    let code = r#"
fn g(p) {
  RT
  READ p
  LT
}

fn f(p) {
  call g(p)
  STOR p
}

fn main() {
  READ x
  call f(x)
}
"#;
    let program = <RecTmIrMachine as Machine>::parse_code(code).unwrap();
    let tape = <RecTmIrMachine as Machine>::parse_ainput("-|a|b").unwrap();
    let mut machine = RecTmIrMachine::make(program, tape).unwrap();

    let _ = run_until_halt(&mut machine, 64).unwrap();
    let head = head_text(machine.current()).unwrap();
    assert_eq!(head, "a");
}

#[test]
fn nested_loop_breaks_resolve() {
    let code = r#"
fn main() {
  loop A: {
    loop B: {
      RT
      READ v
      if v == b break B
    }
    RT
    READ v
    if v == c break A
  }
}
"#;
    let program = <RecTmIrMachine as Machine>::parse_code(code).unwrap();
    let tape = <RecTmIrMachine as Machine>::parse_ainput("-|a|b,c").unwrap();
    let mut machine = RecTmIrMachine::make(program, tape).unwrap();

    let _ = run_until_halt(&mut machine, 64).unwrap();
    let head = head_text(machine.current()).unwrap();
    assert_eq!(head, "c");
}

#[test]
fn repeated_calls_keep_env_isolated() {
    let code = r#"
fn writer(x) {
  READ x
  RT
  STOR x
  LT
}

fn main() {
  READ a
  call writer(a)
  RT
  READ b
  call writer(b)
  LT
  STOR a
}
"#;
    let program = <RecTmIrMachine as Machine>::parse_code(code).unwrap();
    let tape = <RecTmIrMachine as Machine>::parse_ainput("-|a|b,c").unwrap();
    let mut machine = RecTmIrMachine::make(program, tape).unwrap();

    let output = run_until_halt(&mut machine, 128).unwrap();
    assert_eq!(output.print().trim(), "a = a\nb = a");
    let head = head_text(machine.current()).unwrap();
    assert_eq!(head, "a");
}

fn collect_labels(stmts: &[Stmt], labels: &mut Vec<String>) {
    for stmt in stmts {
        if let Stmt::Loop { label, body } = stmt {
            labels.push(label.clone());
            collect_labels(body, labels);
        }
    }
}

fn has_call(stmts: &[Stmt]) -> bool {
    stmts.iter().any(|stmt| match stmt {
        Stmt::Call { .. } => true,
        Stmt::Loop { body, .. } => has_call(body),
        _ => false,
    })
}

#[test]
fn flatten_renames_vars_and_labels() {
    let code = r#"
fn f(x) {
  loop L: {
    READ x
    if x == a break L
  }
}

fn main() {
  loop L: {
    READ x
    if x == b break L
  }
  call f(x)
}
"#;
    let program = <RecTmIrMachine as Machine>::parse_code(code).unwrap();
    let flat = flatten_program(&program).unwrap();
    let main = flat.functions.get("main").unwrap();

    assert!(!has_call(&main.body));

    let mut labels = Vec::new();
    collect_labels(&main.body, &mut labels);
    let mut uniq = labels.clone();
    uniq.sort();
    uniq.dedup();
    assert_eq!(labels.len(), uniq.len());
}
