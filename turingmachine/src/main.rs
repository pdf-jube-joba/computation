use web_sys::{HtmlInputElement, Element};
use yew::html::Scope;
use yew::prelude::*;
use yew::{Properties};

mod machine;
use machine::*;
mod control;
use control::*;

// #[derive(Debug, Clone, PartialEq)]
// enum MoveTo {
//     Right,
//     Left
// }

// impl TryFrom<&str> for MoveTo {
//     type Error = ();
//     fn try_from(value: &str) -> Result<Self, Self::Error> {
//         match value {
//             "R" => Ok(MoveTo::Right),
//             "L" => Ok(MoveTo::Left),
//             _ => Err(()),
//         }
//     }
// }

// type Sign = Option<String>;
// fn sign_to_str(sign: &Sign) -> &str {
//         if let Some(ref str) = sign {str} else {" "}
// }

// #[derive(Debug, Default, Clone, PartialEq, Properties)]
// struct Tape {
//     left: Vec<Sign>,
//     head: Sign,
//     right: Vec<Sign>
// }

// impl Tape {
//     fn move_to(&mut self, m: &MoveTo) {
//         match m {
//             MoveTo::Left => {
//                 let next_head = self.left.pop().unwrap_or_default();
//                 let old_head = std::mem::replace(&mut self.head, next_head);
//                 self.right.push(old_head);
//             }
//             MoveTo::Right => {
//                 let next_head = self.right.pop().unwrap_or_default();
//                 let old_head = std::mem::replace(&mut self.head, next_head);
//                 self.left.push(old_head);
//             }
//         }
//     }
// }

// type State = String;

// type CodeKey = (Sign, State);
// type CodeValue = (Sign, State, MoveTo);

// type Code = HashMap<CodeKey, CodeValue>;

// #[derive(Debug, Default, Clone, PartialEq, Properties)]
// struct TuringMachine {
//     state: State,
//     tape: Tape,
//     code: Code
// }

// impl TuringMachine {
//     pub fn step(&mut self) -> bool {
//         let now = (self.tape.head.clone(), self.state.clone());
//         let next = self.code.get(&now);
//         if let Some((write_sign, next_state, move_to)) = next {
//             self.state = next_state.clone();
//             self.tape.head = write_sign.clone();
//             self.tape.move_to(move_to);
//             true
//         } else {false}
//     }
// }



// struct TuringMachineView;

// #[derive(Clone, PartialEq, Properties)]
// struct TuringMachineProp {
//     machine: TuringMachine,
//     step_callback: Callback<AppMsg>,
// }

// impl Component for TuringMachineView {
//     type Message = ();
//     type Properties = TuringMachineProp;
//     fn create(_ctx: &Context<Self>) -> Self {
//         Self {}
//     }
//     fn view(&self, ctx: &Context<Self>) -> Html {
//         let TuringMachineProp {machine, step_callback} = ctx.props();
//         let callback = step_callback.reform(move |_| {
//             AppMsg::Step
//         });
//         html! {
//             <div class="turing-machine-view">
//                 <p> {"state:"} {machine.state.clone()} {""} </p>
//                 <p> {"l:"} {
//                     machine.tape.left.iter().map(|sign| html!{<> {sign_to_str(sign)} {"|"} </>}).collect::<Html>()
//                 } {"..."} </p>
//                 <p> {"h:"} {
//                     machine.tape.head.clone()
//                 } </p>
//                 <p> {"r:"} {
//                     machine.tape.left.iter().map(|sign| html!{<> {sign_to_str(sign)} {"|"} </>}).collect::<Html>()
//                 } {"..."} </p>
//                 <div class="code-view-entry">
//                     <table>
//                     <thead> <tr>
//                         <td> {"key_sign"} </td>
//                         <td> {"key_state"} </td>
//                         <td> {"value_sign"} </td>
//                         <td> {"value_state"} </td>
//                         <td> {"value_move"} </td>
//                     </tr> </thead>
//                     <tbody>
//                     {
//                         machine.code.iter().map(|((key_sign, key_state), (value_sign, value_state, value_move))|{
//                             html! {
//                                 <tr>
//                                     <td> {sign_to_str(&key_sign)} </td>
//                                     <td> {key_state} </td>
//                                     <td> {sign_to_str(&value_sign)} </td>
//                                     <td> {value_state} </td>
//                                     <td> {format!("{:?}", value_move)} </td>
//                                 </tr>
//                             }
//                         }).collect::<Html>()
//                     }
//                     </tbody>
//                     </table>
//                 </div>
//                 <button onclick={callback}> {"step"} </button>
//             </div>
//         }
//     }
// }

// struct CodeWriteView {
//     code_key_value: String,
//     // input_callback: Callback<String>,
// }

// #[derive(Default, Clone, PartialEq, Properties)]
// struct CodeWriteProps {
//     code_entry: Vec<(CodeKey, CodeValue, Callback<usize>)>,
//     add_callback: Callback<String>,
// }

// enum CodeWriteMsg {
//     ChangedInput(String),
// }

// impl Component for CodeWriteView {
//     type Message = CodeWriteMsg;
//     type Properties = CodeWriteProps;
//     fn create(_ctx: &Context<Self>) -> Self {
//         Self {code_key_value: String::new()}
//     }
//     fn view(&self, ctx: &Context<Self>) -> Html {
//         let CodeWriteProps {code_entry, add_callback} = ctx.props();
//         let str: String = self.code_key_value.to_owned();
//         let add_callback_button: Callback<MouseEvent> = add_callback.reform(move |_e: MouseEvent| {
//             str.to_string()
//         });
//         let message_callback: Callback<CodeWriteMsg> = ctx.link().callback(|e| e);
//         let change_callback: Callback<Event> = message_callback.reform(|e: Event|{
//             let value: HtmlInputElement = e.target_unchecked_into();
//             let str = value.value();
//             CodeWriteMsg::ChangedInput(str)
//         });
//         //  Callback::from(|e: Event|{
//         //     let value: HtmlInputElement = e.target_unchecked_into();
//         //     let value = value.value();
//         //     ctx.link().callback(function)
//         // });
//         html!{
//             <>
//             <div class="codewrite-entry-view">
//                 <table>
//                 <thead> <tr>
//                     <td> {"key_sign"} </td>
//                     <td> {"key_state"} </td>
//                     <td> {"value_sign"} </td>
//                     <td> {"value_state"} </td>
//                     <td> {"value_move"} </td>
//                     <td> </td>
//                 </tr> </thead>
//                 <tbody>
//                 {
//                     code_entry.iter().enumerate()
//                     .map(|(index, ((key_sign, key_state), (value_sign, value_state, value_move), callback))|{
//                         let remove_callback: Callback<MouseEvent> = callback.reform(move |_| index);
//                         html! {
//                             <tr>
//                                 <td> {sign_to_str(&key_sign)} </td>
//                                 <td> {key_state} </td>
//                                 <td> {sign_to_str(&value_sign)} </td>
//                                 <td> {value_state} </td>
//                                 <td> {format!("{:?}", value_move)} </td>
//                                 <td onclick={remove_callback}> {"-"} </td>
//                             </tr>
//                         }
//                     }).collect::<Html>()
//                 }
//                 {
//                     html! {
//                         <>
//                         <input onchange={change_callback}/>
//                         <div onclick={add_callback_button}> {"+"} </div>
//                         </>
//                     }
//                 }
//                 </tbody>
//             </table>
//             </div>
//         </>
//         }
//     }
//     fn update(&mut self, _ctx: &Context<Self>, msg: Self::Message) -> bool {
//         match msg {
//             CodeWriteMsg::ChangedInput(str) => {
//                 self.code_key_value = str;
//             }
//         }
//         true
//     }
// }

// struct EventView;

// #[derive(Debug, Clone, PartialEq, Properties)]
// struct EventProps {
//     event_print: Vec<String>,
// }

// impl Component for EventView {
//     type Message = ();
//     type Properties = EventProps;
//     fn create(_ctx: &Context<Self>) -> Self {
//         Self {}
//     }
//     fn view(&self, ctx: &Context<Self>) -> Html {
//         let EventProps { event_print } = ctx.props();
//         html!{
//             <div class="event-view"> {"event-view"} <br/> {
//                 {
//                     event_print.iter()
//                     .map(|str|{
//                         html!{
//                             <>
//                                 {str} <br/>
//                             </>
//                         }
//                     }).collect::<Html>()
//                 }
//             } </div>
//         }
//     }
// }

// #[derive(Default)]
// struct App {
//     machine: TuringMachine,
//     code_entry: Vec<(CodeKey, CodeValue)>,
//     event_print: Vec<String>,
// }

// #[derive(Debug)]
// enum AppMsg {
//     RemoveRule(usize),
//     AddRule(String),
//     Load,
// }

// impl Component for App {
//     type Message = AppMsg;
//     type Properties = ();
//     fn create(_ctx: &Context<Self>) -> Self {
//         Self::default()
//     }
//     fn view(&self, ctx: &Context<Self>) -> Html {
//         let this_root: Element = todo!();
//         let add_callback = ctx.link().callback(|s| AppMsg::AddRule(s));
//         let remove_callback = ctx.link().callback(AppMsg::RemoveRule);
//         let code_entry_prop: Vec<(CodeKey, CodeValue, Callback<usize>)> = self.code_entry
//             .iter()
//             .map(|(s1, s2)|
//                 (s1.clone(), s2.clone(), remove_callback.clone())
//             ).collect();
//         // let a = html! { <TuringMachineView/> };
//         let turing_machine_handle = yew::Renderer::<TuringMachineView>::with_root(this_root).render();
//         let html = 
//         html! {
//             <>
//             <CodeWriteView code_entry={code_entry_prop} add_callback={add_callback}/>
//             <TuringMachineView/>
//             <EventView event_print={self.event_print.clone()}/>
//             </>
//         };
//         html.
//     }
//     fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
//         self.event_print.push(format!("event: {msg:?}"));
//         match msg {
//             AppMsg::AddRule(s) => {
//                 if let Some((code_key, code_value)) = try_parse(s) {
//                     self.event_print.push("succed to read".to_owned());
//                     self.code_entry.push((code_key, code_value));
//                 } else {
//                     self.event_print.push("failed to read".to_owned());
//                 }
//             }
//             AppMsg::Load => {
                
//             }
//             _ => {}
//         }
//         true
//     }
// }

fn main() {
    let document = gloo::utils::document();
    let machine_element = document.query_selector("machine").unwrap().unwrap();
    let machine_handle = yew::Renderer::<TuringMachineView>::with_root(machine_element).render();

    let control_element = document.query_selector("control").unwrap().unwrap();
    let control_handle = yew::Renderer::<ControlView>::with_root(control_element).render();
}