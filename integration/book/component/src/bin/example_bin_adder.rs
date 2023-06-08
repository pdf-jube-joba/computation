use turing_machine::{machine::*, view::machine::*};
use yew::prelude::*;

fn bin_adder() -> TuringMachineSet {
    unimplemented!()
}

struct App {
    machine: TuringMachineSet,
}

impl Component for App {
    type Message = ();
    type Properties = ();
    fn create(_ctx: &Context<Self>) -> Self {
        App { machine: bin_adder() }
    }
    fn view(&self, _ctx: &Context<Self>) -> Html {
        let tape = self.machine.now_tape();
        html!{
            <TapeView tape={tape}/>
        }
    }
}

fn main() {
    let document = gloo::utils::document();
    let target_element = document.get_element_by_id("example_bin_adder").unwrap();
    yew::Renderer::<App>::with_root(target_element).render();
}
