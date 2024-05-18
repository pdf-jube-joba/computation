use recursive_function_to_turing_machine::compile::*;
use turing_machine_view::machine::*;
use utils::view::*;
use yew::Callback;

fn main() {
    let document = gloo::utils::document();
    let element = document
        .get_element_by_id("recursive_function_to_turing_machine_playground")
        .unwrap();

    let machine_element = document.create_element("div").unwrap();
    element.append_child(&machine_element).unwrap();
    let machine_handle = yew::Renderer::<TuringMachineView>::with_root(machine_element).render();

    let eventlog_handle = yew::Renderer::<EventLogView>::with_root(element.clone()).render();
    let event_log_callback = eventlog_handle.callback(|log| EventLogMsg::Log(log));

    let on_load = Callback::from(
        move |code: String| match recursive_function::manipulation::parse(&code) {
            Ok(fnc) => {
                let builder = compile(&fnc);
                match builder.build() {
                    Ok(machine) => {
                        machine_handle
                            .send_message(TuringMachineMsg::LoadFromMachine(Box::new(machine)));
                    }
                    Err(err) => {
                        event_log_callback.emit(format!("{:?}", err));
                    }
                }
            }
            Err(err) => event_log_callback.emit(err),
        },
    );

    let control_element = document.create_element("div").unwrap();
    element.append_child(&control_element).unwrap();
    let _control_handle = yew::Renderer::<utils::view::CodeView>::with_root_and_props(
        control_element,
        utils::view::CodeProps { on_load },
    )
    .render();
}
