// pub struct NatNumInterpretation;

// impl NatNumInterpretation {
//     fn partition() -> Sign {
//         Sign::try_from("-").unwrap()
//     }
//     fn one() -> Sign {
//         Sign::try_from("1").unwrap()
//     }
//     pub fn interpretation() -> Interpretation<String, String> {
//         fn write(input: String) -> Result<TapeAsVec, String> {
//             let right: Vec<Sign> = NumberTuple::try_from(input)?.0
//                 .into_iter()
//                 .flat_map(|Number(num)| {
//                         std::iter::repeat(NatNumInterpretation::one())
//                         .take(num)
//                         .chain(std::iter::once(NatNumInterpretation::partition()))
//                 })
//                 .collect();

//             Ok(TapeAsVec::new(
//                 Vec::new(),
//                 NatNumInterpretation::partition(),
//                 right,
//             ))
//         }
//         fn read(tape: TapeAsVec) -> Result<String, String> {
//             let mut vec = Vec::new();
//             let right = tape.right.clone();
            
//             let mut num = 0;
//             for i in 0..right.len() {
//                 match right[i] {
//                     _ if right[i] == NatNumInterpretation::partition() => {
//                         vec.push(Number(num))
//                     }
//                     _ if right[i] == NatNumInterpretation::one() => {
//                         num += 1;
//                     }
//                     _ if right[i] == Sign::blank() => {
//                         break;
//                     }
//                     _ => unreachable!()
//                 }
//             }
//             Ok(NumberTuple(vec).into())
//         }
//         Interpretation::new(
//             write,
//             read
//         )
//     }
// }

// pub fn inc() -> TuringMachineBuilder<String, String> {
//     let mut builder = TuringMachineBuilder::new("increment", NatNumInterpretation::interpretation()).unwrap();
//     builder
//         .init_state(State::try_from("start").unwrap())
//         .accepted_state(vec![
//             State::try_from("end").unwrap()
//         ])
//         // .code_push(" , start_inc , , end_inc , C").unwrap()
//         .code_push_str("-, start, -, read, R").unwrap()
//         .code_push_str("1, read, 1, read, R").unwrap()
//         .code_push_str("-, read, 1, write, R").unwrap()
//         .code_push_str(" , write, -, write_end, L").unwrap()
//         .code_push_str("1, write_end, 1, write_end, L").unwrap()
//         .code_push_str("-, write_end, - , end, C").unwrap()
//         ;
//     builder
// }