use machine::MachineMsg;
use web_sys::Element;
use web_sys::HtmlInputElement;
use yew::html::Scope;
use yew::prelude::*;
use yew::Properties;

use utils::view::*;

use logic_circuit::machine::LoC;
use logic_circuit::manipulation;

pub mod control;
pub mod machine;

pub fn playground(element: Element) {
    let machine_handle = yew::Renderer::<machine::MachineView>::with_root(element.clone()).render();
    let load_machine_callback =
        machine_handle.callback(|lc| MachineMsg::LoadFromMachine(Box::new(lc)));
    let eventlog_handle = yew::Renderer::<EventLogView>::with_root(element.clone()).render();
    let event_log_callback = eventlog_handle.callback(|log| EventLogMsg::Log(log));
    let on_load = Callback::from(move |code: String| match manipulation::parse(&code) {
        Ok(lc) => {
            load_machine_callback.emit(lc);
        }
        Err(err) => event_log_callback.emit(format!("{err:?}")),
    });
    let code_handle =
        yew::Renderer::<CodeView>::with_root_and_props(element, CodeProps { on_load });
}
