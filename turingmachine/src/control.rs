use yew::html::Scope;
use yew::prelude::*;
use yew::{Properties};
use web_sys::{HtmlInputElement};

use super::machine::*;
struct CodeWriteView {
  code_key_value: String,
}

#[derive(Default, Clone, PartialEq, Properties)]
struct CodeWriteProps {
  code_entry: Vec<(CodeKey, CodeValue, Callback<usize>)>,
  add_callback: Callback<String>,
}

enum CodeWriteMsg {
  ChangedInput(String),
}

impl Component for CodeWriteView {
  type Message = CodeWriteMsg;
  type Properties = CodeWriteProps;
  fn create(_ctx: &Context<Self>) -> Self {
      Self {code_key_value: String::new()}
  }
  fn view(&self, ctx: &Context<Self>) -> Html {
      let CodeWriteProps {code_entry, add_callback} = ctx.props();
      let str: String = self.code_key_value.to_owned();
      let add_callback_button: Callback<MouseEvent> = add_callback.reform(move |_e: MouseEvent| {
          str.to_string()
      });
      let message_callback: Callback<CodeWriteMsg> = ctx.link().callback(|e| e);
      let change_callback: Callback<Event> = message_callback.reform(|e: Event|{
          let value: HtmlInputElement = e.target_unchecked_into();
          let str = value.value();
          CodeWriteMsg::ChangedInput(str)
      });
      //  Callback::from(|e: Event|{
      //     let value: HtmlInputElement = e.target_unchecked_into();
      //     let value = value.value();
      //     ctx.link().callback(function)
      // });
      html!{
          <>
          <div class="codewrite-entry-view">
              <table>
              <thead> <tr>
                  <td> {"key_sign"} </td>
                  <td> {"key_state"} </td>
                  <td> {"value_sign"} </td>
                  <td> {"value_state"} </td>
                  <td> {"value_move"} </td>
                  <td> </td>
              </tr> </thead>
              <tbody>
              {
                  code_entry.iter().enumerate()
                  .map(|(index, ((key_sign, key_state), (value_sign, value_state, value_move), callback))|{
                      let remove_callback: Callback<MouseEvent> = callback.reform(move |_| index);
                      html! {
                          <tr>
                              <td> {sign_to_str(&key_sign)} </td>
                              <td> {key_state} </td>
                              <td> {sign_to_str(&value_sign)} </td>
                              <td> {value_state} </td>
                              <td> {format!("{:?}", value_move)} </td>
                              <td onclick={remove_callback}> {"-"} </td>
                          </tr>
                      }
                  }).collect::<Html>()
              }
              {
                  html! {
                      <>
                      <input onchange={change_callback}/>
                      <div onclick={add_callback_button}> {"+"} </div>
                      </>
                  }
              }
              </tbody>
          </table>
          </div>
      </>
      }
  }
  fn update(&mut self, _ctx: &Context<Self>, msg: Self::Message) -> bool {
      match msg {
          CodeWriteMsg::ChangedInput(str) => {
              self.code_key_value = str;
          }
      }
      true
  }
}

struct EventView;

#[derive(Debug, Clone, PartialEq, Properties)]
struct EventProps {
    event_print: Vec<String>,
}

impl Component for EventView {
    type Message = ();
    type Properties = EventProps;
    fn create(_ctx: &Context<Self>) -> Self {
        Self {}
    }
    fn view(&self, ctx: &Context<Self>) -> Html {
        let EventProps { event_print } = ctx.props();
        html!{
            <div class="event-view"> {"event-view"} <br/> {
                {
                    event_print.iter()
                    .map(|str|{
                        html!{
                            <>
                                {str} <br/>
                            </>
                        }
                    }).collect::<Html>()
                }
            } </div>
        }
    }
}

#[derive(Default)]
pub struct ControlView {
    machine: Option<Scope<TuringMachineView>>,
}

pub enum ControlMsg {
    SetTargetMachineView(Scope<TuringMachineView>),
    EventLog(String),
}

#[derive(Clone, PartialEq, Properties)]
pub struct ControlProp {}

impl Component for ControlView {
  type Message = ControlMsg;
  type Properties = ();
  fn create(_ctx: &Context<Self>) -> Self {
    Self::default()
  }
  fn view(&self, _ctx: &Context<Self>) -> Html {
      html!{
            <>
            {"hello"}
            </>
      }
  }
    fn update(&mut self, _ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            ControlMsg::SetTargetMachineView(scope) => {
                self.machine = Some(scope);
            }
            ControlMsg::EventLog(str) => {
                todo!()
            }
        }
        true
    }
}