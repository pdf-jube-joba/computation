use turing_machine::{machine::*, view::machine::*, manipulation::*};
use yew::prelude::*;
use gloo::timers::callback::Interval;

fn bin_adder() -> TuringMachineSet {
    let code =  code::parse_code(include_str!("bin_adder.txt")).unwrap();
    let tape_input = "|-|1 0 1 0 1 -".to_string();
    let mut builder = TuringMachineBuilder::new("bin_adder", tape::string_split_by_bar_interpretation()).unwrap();
    builder
        .code_new(code)
        .init_state(State::try_from("start").unwrap())
        .accepted_state(vec![State::try_from("end").unwrap()])
        .input(tape_input);
    builder.build().unwrap()
}

struct App {
    machine: TuringMachineSet,
    tick_active: bool,
    #[allow(dead_code)]
    tick_interval: Interval,
}

#[derive(Clone, PartialEq)]
enum Msg {
    Step(usize),
    Tick,
    ToggleAuto,
}

impl Component for App {
    type Message = Msg;
    type Properties = ();
    fn create(ctx: &Context<Self>) -> Self {
        let callback = ctx.link().callback(|_| Msg::Tick);
        let interval = Interval::new(1000, move || callback.emit(()));
        App { 
            machine: bin_adder(),
            tick_active: false,
            tick_interval: interval,
        }
    }
    fn view(&self, ctx: &Context<Self>) -> Html {
        let callback_step_usr = ctx.link().callback(|u| Msg::Step(u));
        let callback_toggle = ctx.link().callback(|_:()| Msg::ToggleAuto);
        html!{
            <MachineView
                callback_step_usr={callback_step_usr}
                callback_toggle_autostep={callback_toggle}
                now_toggle_state={self.tick_active}
                machine={self.machine.clone()}
                code_visible={false}
            />
        }
    }
    fn update(&mut self, _ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Msg::Step(u) => {
                let res = self.machine.step(u);
                if res.is_err() {
                    self.tick_active = false;
                }
            }
            Msg::ToggleAuto => {
                self.tick_active = !self.tick_active;
            }
            Msg::Tick => {
                if self.tick_active {
                    let res = self.machine.step(1);
                    if res.is_err() {
                        self.tick_active = false;
                    }
                }
            }
        }
        true
    }
}

fn main() {
    let document = gloo::utils::document();
    let target_element = document.get_element_by_id("example_bin_adder").unwrap();
    yew::Renderer::<App>::with_root(target_element).render();
}
