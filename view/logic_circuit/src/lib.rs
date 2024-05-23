use std::collections::HashMap;

use logic_circuit::machine::{LoC, Name};
// use machine::MachineMsg;
use web_sys::Element;
use web_sys::HtmlInputElement;
use yew::html::Scope;
use yew::prelude::*;
use yew::Properties;

use utils::view::*;

use logic_circuit::manipulation;

pub mod machine;

pub fn playground(element: Element) {
    // let eventlog_handle = yew::Renderer::<EventLogView>::with_root(element.clone()).render();
    // let event_log_callback = eventlog_handle.callback(|log| EventLogMsg::Log(log));

    // let machine_handle = yew::Renderer::<machine::MachineView>::with_root(element.clone()).render();
    // let load_machine_callback =
    //     machine_handle.callback(|lc| MachineMsg::LoadFromMachine(Box::new(lc)));


    // machine_handle.send_message(MachineMsg::SetEventLog(event_log_callback.clone()));

    // let on_load = Callback::from(move |code: String| match manipulation::parse_main(&code) {
    //     Ok(lc) => {
    //         load_machine_callback.emit(lc);
    //     }
    //     Err(err) => event_log_callback.emit(format!("{err:?}")),
    // });
    // let _code_handle = yew::Renderer::<utils::view::CodeView>::with_root_and_props(
    //     element,
    //     utils::view::CodeProps { on_load },
    // )
    // .render();
}

pub fn playground_with(element: Element, maps: HashMap<Name, LoC>) {
    // let machine_handle = yew::Renderer::<machine::MachineView>::with_root(element.clone()).render();
    // let load_machine_callback =
    //     machine_handle.callback(|lc| MachineMsg::LoadFromMachine(Box::new(lc)));

    // let eventlog_handle = yew::Renderer::<EventLogView>::with_root(element.clone()).render();
    // let event_log_callback = eventlog_handle.callback(|log| EventLogMsg::Log(log));

    // machine_handle.send_message(MachineMsg::SetEventLog(event_log_callback.clone()));

    // let on_load = Callback::from(move |code: String| {
    //     match manipulation::parse_main_with_maps(&code, &maps) {
    //         Ok(lc) => {
    //             load_machine_callback.emit(lc);
    //         }
    //         Err(err) => event_log_callback.emit(format!("{err:?}")),
    //     }
    // });
    // let _code_handle = yew::Renderer::<utils::view::CodeView>::with_root_and_props(
    //     element,
    //     utils::view::CodeProps { on_load },
    // )
    // .render();
}

pub fn set_machine(element: Element, lc: LoC) {
    // let machine_handle = yew::Renderer::<machine::MachineView>::with_root(element).render();
    // machine_handle.send_message(MachineMsg::LoadFromMachine(Box::new(lc)));
}
