use crate::machine::{Sign, Tape, Direction};

fn map_v(v: Vec<&str>) -> Vec<Sign> {
    v.into_iter().map(|s| s.parse().unwrap()).collect()
}

#[test]
fn test_tape_eq() {

    // every thing is the same  ..., "", ["-"], "-", "", ...
    let v1 = Tape::from_vec(map_v(vec!["-", "-"]), 0);
    let v2 = Tape::from_vec(map_v(vec!["-", "-", ""]), 0);
    let v3 = Tape::from_vec(map_v(vec!["", "-", "-"]), 1);
    assert!(v1.eq(&v2));
    assert!(v1.eq(&v3));
}

#[test]
fn test_tape_lr() {
    let v = map_v(vec!["0", "1", "2", "3", "4", "5", "6"]);

    let mut tape = Tape::from_vec(v.clone(), 3).unwrap();
    tape.move_to(&Direction::Left);
    let tape2 = Tape::from_vec(v.clone(), 2).unwrap();
    assert!(tape.eq(&tape2));
}
