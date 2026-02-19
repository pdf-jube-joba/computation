use utils::{Machine, TextCodec};

use super::{Environment, RecTmIrMachine, Snapshot};

fn run_until_halt(machine: &mut RecTmIrMachine, limit: usize) -> EnvironmentResult {
    for _ in 0..limit {
        if let Some(env) = machine.step(())? {
            return Ok(env);
        }
    }
    Err("step limit exceeded".to_string())
}

type EnvironmentResult = Result<Environment, String>;

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
alphabet: (a, b)
fn f() {
  label entry: {
    READ x
  }
}

fn main() {
  label entry: {
    READ x
    RT
    call f
    LT
    STOR x
  }
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
    READ mark
    jump scan
  }
  label scan: {
    RT
    READ cur
    jump_if cur == x : done
    jump scaｌ
  }
  label done: {
    STOR mark
  }
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
alphabet: (a, b)
fn g() {
  label entry: {
    RT
    READ p
    LT
  }
}

fn f() {
  label entry: {
    call g
    STOR p
  }
}

fn main() {
  label entry: {
    READ x
    call f
  }
}
"#;
    let program = <RecTmIrMachine as Machine>::parse_code(code).unwrap();
    let tape = <RecTmIrMachine as Machine>::parse_ainput("-|a|b").unwrap();
    let mut machine = RecTmIrMachine::make(program, tape).unwrap();

    let _ = run_until_halt(&mut machine, 64).unwrap();
    let head = head_text(machine.current()).unwrap();
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
    READ v
    jump_if v == b : scan_c
    jump scan_b
  }
  label scan_c: {
    RT
    READ v
    jump_if v == c : done
    jump scan_b
  }
  label done: {
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
alphabet: (a, b, c)
fn writer() {
  label entry: {
    READ x
    RT
    STOR x
    LT
  }
}

#[test]
fn break_jumps_to_next_block() {
    let code = r#"
alphabet: (a, b)
fn main() {
  label first: {
    STOR const a
    break
    STOR const b
  }
  label second: {
    RT
    STOR const b
  }
}
"#;
    let program = <RecTmIrMachine as Machine>::parse_code(code).unwrap();
    let tape = <RecTmIrMachine as Machine>::parse_ainput("-|-").unwrap();
    let mut machine = RecTmIrMachine::make(program, tape).unwrap();

    let _ = run_until_halt(&mut machine, 16).unwrap();
    let head = head_text(machine.current()).unwrap();
    assert_eq!(head, "b");
}

#[test]
fn continue_loops_current_block() {
    let code = r#"
alphabet: (a)
fn main() {
  label entry: {
    STOR const a
    continue
    STOR const -
  }
  label next: {
    STOR const -
  }
}
"#;
    let program = <RecTmIrMachine as Machine>::parse_code(code).unwrap();
    let tape = <RecTmIrMachine as Machine>::parse_ainput("-|-").unwrap();
    let mut machine = RecTmIrMachine::make(program, tape).unwrap();

    let result = run_until_halt(&mut machine, 8);
    assert!(result.is_err());
}
