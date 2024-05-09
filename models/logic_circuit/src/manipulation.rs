// use std::collections::{HashMap, HashSet};

// use crate::machine::{circuit_components::*, logic_circuit::*};
// use anyhow::{bail, Result};
// use either::Either;

// fn parse_one(code: &str) -> Result<ExtensibleLogicCircuit> {
//     let mut gates: HashMap<
//         String,
//         (
//             Either<(Label, Option<Bool>), ExtensibleLogicCircuit>,
//             Vec<String>,
//         ),
//     > = HashMap::new();
//     for l in code.lines() {
//         let l: Vec<_> = l.split_whitespace().collect();
//         let name = l[0].to_string();
//         let gate: Either<(Label, Option<Bool>), ExtensibleLogicCircuit> = match l[1] {
//             "IN" if l.len() == 2 => Either::Left((Label::InOut(InOutLabel::Input), None)),
//             "OUT" if l.len() == 2 => Either::Left((Label::InOut(InOutLabel::Output), None)),
//             "BR" if l.len() == 4 => {
//                 Either::Left((Label::Control(ControlLabel::Branch), Some(l[2].parse()?)))
//             }
//             "NOT" if l.len() == 4 => {
//                 Either::Left((Label::Logic(LogicLabel::Not), Some(l[2].parse()?)))
//             }
//             "OR" if l.len() == 5 => {
//                 Either::Left((Label::Logic(LogicLabel::Not), Some(l[2].parse()?)))
//             }
//             "AND" if l.len() == 5 => {
//                 Either::Left((Label::Logic(LogicLabel::Not), Some(l[2].parse()?)))
//             }
//             "C0" if l.len() == 3 => {
//                 Either::Left((Label::Logic(LogicLabel::Not), Some(l[2].parse()?)))
//             }
//             _ => {
//                 let l = parse_iter(code)?;
//                 Either::Right(l)
//             }
//         };
//         let froms = l[3..].iter().map(|s| s.to_string()).collect();
//         gates.insert(name, (gate, froms));
//     }
    
//     todo!()
// }

// fn parse_iter(code: &str) -> Result<ExtensibleLogicCircuit> {
//     todo!()
// }
