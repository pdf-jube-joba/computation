use crate::example::utils_map;
use crate::machine::*;
use crate::parse::parse_main;
use utils::{bool::Bool, identifier::Identifier};

fn pin(name: &str, pin: &str) -> (Identifier, Identifier) {
    (
        Identifier::new(name).unwrap(),
        Identifier::new(pin).unwrap(),
    )
}

fn signal(items: Vec<(NamedPin, Bool)>) -> Signal {
    Signal::new(items)
}

#[test]
fn test_example() {
    let list = utils_map();
    let lc = list.get(&Identifier::new("one-shot").unwrap()).unwrap();
    eprintln!("{:?}", lc);
    eprintln!("{:?}", lc.as_graph_group());
    eprintln!("{:?}", lc.get_inpins());
}

#[test]
fn test_xor() {
    let list = utils_map();
    let mut lc = list.get(&Identifier::new("XOR").unwrap()).unwrap().clone();
    eprintln!("{:?}", lc.get_otputs());

    let inputs: Vec<(NamedPin, Bool)> = vec![pin("XOR", "IN0"), pin("XOR", "IN1")]
        .into_iter()
        .map(|p| (p, Bool::F))
        .collect();
    for _ in 0..6 {
        lc.step(signal(inputs.clone()));
        eprintln!("{:?}", lc.get_otputs());
    }

    eprintln!("----");

    let inputs: Vec<(NamedPin, Bool)> = vec![pin("XOR", "IN0"), pin("XOR", "IN1")]
        .into_iter()
        .map(|p| (p, Bool::T))
        .collect();
    for _ in 0..6 {
        lc.step(signal(inputs.clone()));
        eprintln!("{:?}", lc.get_otputs());
    }

    eprintln!("----");

    let inputs = vec![(pin("XOR", "IN0"), Bool::T), (pin("XOR", "IN1"), Bool::F)];
    for _ in 0..6 {
        lc.step(signal(inputs.clone()));
        eprintln!("{:?}", lc.get_otputs());
    }
}

#[test]
fn test_parse_main() {
    let s = "graph: main {
        in {A, d}
        out {a=b.c}
        A, AND-T {}
      }";
    let _c = parse_main(s).unwrap();
}

#[test]
fn test_gate() {
    let mut gate = Gate {
        kind: GateKind::And,
        state: Bool::F,
    };
    assert_eq!(
        gate.get_inpins(),
        vec![pin("AND", "IN0"), pin("AND", "IN1")]
    );
    assert_eq!(gate.get_otpins(), vec![pin("AND", "OUT")]);
    assert_eq!(gate.get_otputs(), vec![(pin("AND", "OUT"), Bool::F)]);
    gate.step(signal(vec![
        (pin("AND", "IN0"), Bool::T),
        (pin("AND", "IN1"), Bool::T),
    ]));
    assert_eq!(gate.state, Bool::T);
    assert_eq!(gate.get_otputs(), vec![(pin("AND", "OUT"), Bool::T)]);
    gate.step(signal(vec![
        (pin("AND", "IN0"), Bool::T),
        (pin("AND", "IN1"), Bool::F),
    ]));
    assert_eq!(gate.state, Bool::F);
    assert_eq!(gate.get_otputs(), vec![(pin("AND", "OUT"), Bool::F)]);
}

#[test]
fn test_mix() {
    // graph like
    // A.IN0 ---> A <--- A.IN1
    //            |-- A.OUT == B.IN0 ---> B <--- B.IN1
    //                                    |--> B.OUT
    // B.OUT takes 2 step to be T if A.IN0 and A.IN1 are T
    // B.OUT takes 1 step to be T is B.IN1 is T
    let mut mix = MixLogicCircuit {
        kind: Identifier::new("MIX").unwrap(),
        verts: vec![
            (
                Identifier::new("A").unwrap(),
                LogicCircuit::Gate(Gate {
                    kind: GateKind::And,
                    state: Bool::F,
                }),
            ),
            (
                Identifier::new("B").unwrap(),
                LogicCircuit::Gate(Gate {
                    kind: GateKind::Or,
                    state: Bool::F,
                }),
            ),
        ],
        edges: vec![(pin("A", "OUT"), pin("B", "IN0"))],
        inpin_maps: vec![],
        otpin_maps: vec![],
    };
    assert_eq!(
        mix.get_inpins(),
        vec![pin("A", "IN0"), pin("A", "IN1"), pin("B", "IN1")]
    );
    assert_eq!(mix.get_otpins(), vec![pin("B", "OUT")]);
    assert_eq!(mix.get_otputs(), vec![(pin("B", "OUT"), Bool::F)]);

    mix.step(signal(vec![(pin("A", "IN0"), Bool::T)]));
    assert_eq!(mix.get_otputs(), vec![(pin("B", "OUT"), Bool::F)]);

    mix.step(signal(vec![(pin("A", "IN0"), Bool::T)]));
    assert_eq!(mix.get_otputs(), vec![(pin("B", "OUT"), Bool::F)]);

    // A.IN0 and A.IN1 are T
    mix.step(signal(vec![
        (pin("A", "IN0"), Bool::T),
        (pin("A", "IN1"), Bool::T),
        // B.IN1 is F if not set
    ]));
    assert_eq!(mix.get_otputs(), vec![(pin("B", "OUT"), Bool::F)]);
    mix.step(signal(vec![
        (pin("A", "IN0"), Bool::F),
        (pin("A", "IN1"), Bool::F),
        // B.IN1 is F if not set
    ]));
    assert_eq!(mix.get_otputs(), vec![(pin("B", "OUT"), Bool::T)]);
}
