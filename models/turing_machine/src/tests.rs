use utils::TextCodec;

use crate::machine::{Direction, Sign, Tape};

fn map_v(v: Vec<&str>) -> Vec<Sign> {
    v.iter().map(|s| TextCodec::parse(s).unwrap()).collect()
}

#[test]
fn test_tape_eq() {
    // every thing is the same  ..., "", ["-"], "-", "", ...
    let v1 = Tape::from_vec(map_v(vec!["-", "-"]), 0);
    let v2 = Tape::from_vec(map_v(vec!["-", "-", "-"]), 0);
    let v3 = Tape::from_vec(map_v(vec!["-", "-", "-"]), 1);
    assert!(v1.eq(&v2));
    assert!(v1.eq(&v3));
}

#[test]
fn test_tape_lr() {
    let v = map_v(vec!["a", "b", "c", "d", "e", "f", "g"]);

    let mut tape = Tape::from_vec(v.clone(), 3).unwrap();
    tape.move_to(&Direction::Left);
    let tape2 = Tape::from_vec(v.clone(), 2).unwrap();
    assert!(tape.eq(&tape2));
}

#[test]
fn tape_lr_blank_generated() {
    let v = map_v(vec!["a", "b", "c", "d", "e", "f", "g"]);

    let mut tape = Tape::from_vec(v.clone(), 1).unwrap();
    for _ in 0..3 {
        tape.move_to(&Direction::Left);
    }

    let v2 = map_v(vec!["-", "-", "a", "b", "c", "d", "e", "f", "g"]);

    let tape2 = Tape::from_vec(v2.clone(), 0).unwrap();
    assert!(tape.eq(&tape2));
}

#[test]
fn tape_parse() {
    use utils::TextCodec;
    let _ = Tape::parse("a|b|c").unwrap();
    let _ = Tape::parse("-|-|-").unwrap();
    let _ = Tape::parse("a,b|-|c,d").unwrap();
    let _ = Tape::parse("|-|").unwrap();
}
