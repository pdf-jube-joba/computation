use crate::machine::*;
use crate::manipulation::{TuringMachineBuilder, Interpretation, composition};

#[derive(Debug, Clone, PartialEq)]
pub struct Number(usize);

impl Number {
    fn is_zero(self) -> bool {
        self.0 == 0
    }
    fn succ(self) -> Self {
        Number(self.0 + 1)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct NumberTuple(Vec<Number>);

impl TryFrom<String> for NumberTuple {
    type Error = String;
    fn try_from(value: String) -> Result<Self, Self::Error> {
        todo!()
    }
}

impl Into<String> for NumberTuple {
    fn into(self) -> String {
        todo!()
    }
}

pub struct NatNumInterpretation;

impl NatNumInterpretation {
    fn partition() -> Sign {
        Sign::try_from("-").unwrap()
    }
    fn one() -> Sign {
        Sign::try_from("1").unwrap()
    }
}

impl Interpretation for NatNumInterpretation {
    type Input = NumberTuple;
    type Output = NumberTuple;
    fn write(&self, input: Self::Input) -> Result<TapeAsVec, String> {
        let right: Vec<Sign> = input.0
            .into_iter()
            .flat_map(|num| {
                    std::iter::repeat(NatNumInterpretation::one())
                    .take(num.0)
                    .chain(std::iter::once(NatNumInterpretation::partition()))
            })
            .collect();

        eprintln!("{right:?}");

        Ok(TapeAsVec::new(
            Vec::new(),
            NatNumInterpretation::partition(),
            right,
        ))
    }
    
    fn read(&self, tape: TapeAsVec) -> Result<Self::Output, String> {
        let mut vec = Vec::new();
        let right = tape.right.clone();
        
        let mut num = 0;
        for i in 0..right.len() {
            match right[i] {
                _ if right[i] == NatNumInterpretation::partition() => {
                    vec.push(Number(num))
                }
                _ if right[i] == NatNumInterpretation::one() => {
                    num += 1;
                }
                _ if right[i] == Sign::blank() => {
                    break;
                }
                _ => unreachable!()
            }
        }
        Ok(NumberTuple(vec))
    }
}

fn inc() -> TuringMachineBuilder<NumberTuple, NumberTuple> {
    let mut builder = TuringMachineBuilder::new("one", NatNumInterpretation).unwrap();
    builder
        .init_state(State::try_from("start").unwrap())
        .accepted_state(vec![
            State::try_from("end").unwrap()
        ])
        // .code_push(" , start_inc , , end_inc , C").unwrap()
        .code_push_str("-, start, -, read, R").unwrap()
        .code_push_str("1, read, 1, read, R").unwrap()
        .code_push_str("-, read, 1, write, R").unwrap()
        .code_push_str(" , write, -, write_end, L").unwrap()
        .code_push_str("1, write_end, 1, write_end, L").unwrap()
        .code_push_str("-, write_end, - , end, C").unwrap()
        ;
    builder
}

mod test {
    use super::*;

    #[test]
    fn inc_test1() {
        let number_pred = Number(10);

        let mut builder = inc();
        let mut machine = builder
            .build_input(&NumberTuple(vec![number_pred.clone()])).unwrap();

        machine.step(100);

        let result = machine.result().unwrap();
        assert_eq!(NumberTuple(vec![number_pred.succ()]), result)
    }

    #[test]
    fn inc_test2() {
        let number_pred = Number(10);

        let mut builder = composition::<NatNumInterpretation, NatNumInterpretation, _, _, _>(inc(), State::try_from("end").unwrap(), inc()).unwrap();
        let mut machine = builder
            .build_input(&NumberTuple(vec![number_pred.clone()])).unwrap();

        machine.step(200);
        
        let result = machine.result().unwrap();
        assert_eq!(NumberTuple(vec![number_pred.succ()]), result)
    }
}

pub mod view {
    use yew::html::Scope;
    use yew::prelude::*;
    use crate::{view::{TuringMachineView, TuringMachineMsg}, manipulation};
    use super::*;

    #[derive(Debug, Default)]
    pub struct ExampleView {
        scope: Option<Scope<TuringMachineView>>,
    }
    pub enum ExampleMsg {
        SetTargetMachineView(Scope<TuringMachineView>),
        SendIncMachine,
        SendIncIncMachine,
    }

    #[derive(Debug, Default, Clone, PartialEq, Properties)]
    pub struct ExampleProps {}

    impl Component for ExampleView {
        type Message = ExampleMsg;
        type Properties = ExampleProps;
        fn create(ctx: &Context<Self>) -> Self {
            Self::default()
        }
        fn view(&self, ctx: &Context<Self>) -> Html {
            html!{
                <>
                    {"example"} <br/>
                    <>
                        <button onclick={ctx.link().callback(|_| ExampleMsg::SendIncMachine)}> { "inc 10" } </button>
                    </>
                    <br/>
                    <>
                        <button onclick={ctx.link().callback(|_| ExampleMsg::SendIncIncMachine)}> { "incinc 10" } </button>
                    </>
                    // <button onclick={ctx.link()}> { "zero" } </button>
                </>
            }
        }
        fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
            match msg {
                ExampleMsg::SetTargetMachineView(scope) => {
                    self.scope = Some(scope);
                }
                ExampleMsg::SendIncMachine => {
                    // if let Some(scope) = &self.scope {
                    //     let mut builder = inc();
                    //     builder.write(&vec![Number(10)]).unwrap();
                    //     scope.send_message(TuringMachineMsg::LoadFromMachine(builder.build().unwrap()))
                    // }
                }
                ExampleMsg::SendIncIncMachine => {
                    if let Some(scope) = &self.scope {
                        let mut builder = composition::<NatNumInterpretation, NatNumInterpretation, _, _, _>(inc(), State::try_from("end").unwrap(), inc()).unwrap();
                        builder.input(&NumberTuple(vec![Number(5)]));
                        scope.send_message(TuringMachineMsg::LoadFromBuilder(builder.stringfy()))
                    }
                }
            }
            false
        }
}
}
