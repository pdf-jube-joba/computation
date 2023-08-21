use counter_machine::machine::CounterMachine;
use yew::{html, Component, Properties};

#[derive(Default)]
struct CounterMachineView {
    machine: Option<CounterMachine>,
}

#[derive(Debug, Default, Clone, PartialEq, Properties)]
struct CounterMachineProp {}

struct CounterMachineMsg {}

impl Component for CounterMachineView {
    type Message = CounterMachineMsg;
    type Properties = CounterMachineProp;
    fn create(_ctx: &yew::Context<Self>) -> Self {
        Self::default()
    }
    fn view(&self, _ctx: &yew::Context<Self>) -> yew::Html {
        let html1 = if let Some(machine) = &self.machine {
            let code_html: yew::Html = (machine.code_as_vec())
                .into_iter()
                .enumerate()
                .map(|(i, s)| {
                    let v = if machine.program_counter == i.into() {
                        "selected"
                    } else {
                        "not selected"
                    };
                    html! {
                    <>
                        <div class={v}>
                            {s}
                        </div> <br/>
                    </>}
                })
                .collect();
            html! {
                <>
                    {"machine"}
                    {code_html}
                </>
            }
        } else {
            html! {
                <>
                    {"not found"}
                </>
            }
        };

        html! {
            {html1}
        }
    }
}
