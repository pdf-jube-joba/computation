use lambda_calculus_view::*;
use recursive_function::manipulation;
use recursive_function_to_lambda_calculus::compile;
use recursive_function_view::*;
use utils::view::*;
use yew::Callback;

fn main() {
    let document = gloo::utils::document();
    let element = document
        .get_element_by_id("recursive_function_to_lambda_calculus_playground")
        .unwrap();

    let machine_element = document.create_element("div").unwrap();
    element.append_child(&machine_element).unwrap();
    let machine_handle = yew::Renderer::<LambdaCalculusView>::with_root(machine_element).render();

    let eventlog_handle = yew::Renderer::<EventLogView>::with_root(element.clone()).render();
    let event_log_callback = eventlog_handle.callback(|log| EventLogMsg::Log(log));

    let on_load = Callback::from(move |code: String| match manipulation::parse(&code) {
        Ok(func) => {
            let term = compile(&func);
            machine_handle.send_message(LambdaCalculusMsg::Change(term));
        }
        Err(err) => {
            event_log_callback.emit(err);
        }
    });

    let control_element = document.create_element("div").unwrap();
    element.append_child(&control_element).unwrap();
    let _control_handle = yew::Renderer::<utils::view::CodeView>::with_root_and_props(
        control_element,
        utils::view::CodeProps { on_load },
    )
    .render();
}
