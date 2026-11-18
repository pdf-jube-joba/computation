///|
pub(all) enum Transition[S] {
  Continue(S, String)
  Halt(String)
}

///|
pub struct Model {
  make : (String, String) -> Unit raise @base.Fault
  step : (String) -> String raise @base.Fault
  snapshot : () -> String raise @base.Fault
  restore : (String) -> Unit raise @base.Fault
  render : (String) -> String raise @base.Fault
}

///|
pub fn[S] model(
  make : (String, String) -> S raise @base.Fault,
  step : (S, String) -> Transition[S] raise @base.Fault,
  encode : (S) -> Json,
  decode : (Json) -> S raise @base.Fault,
  render : (S) -> Json,
) -> Model {
  let state : Ref[S?] = { val: None, }
  fn current() -> S raise @base.Fault {
    match state.val {
      Some(value) => value
      None => @base.error("No active machine; call make first")
    }
  }
  {
    make: (code, input) => {
      let value = make(code, input)
      state.val = Some(value)
    },
    step: input => {
      match step(current(), input) {
        Continue(next, output) => {
          state.val = Some(next)
          @base.object([
            ("kind", @base.jstr("continue")),
            ("routput", @base.jstr(output)),
          ]).stringify()
        }
        Halt(output) => {
          state.val = None
          @base.object([
            ("kind", @base.jstr("halt")),
            ("foutput", @base.jstr(output)),
          ]).stringify()
        }
      }
    },
    snapshot: () => encode(current()).stringify(),
    restore: text => {
      let value = decode(@base.parse_json(text))
      state.val = Some(value)
    },
    render: text => render(decode(@base.parse_json(text))).stringify(),
  }
}

///|
pub(all) struct Compiler {
  compile_code : (String) -> String raise @base.Fault
  encode_ainput : (String) -> String raise @base.Fault
  encode_rinput : (String) -> String raise @base.Fault
  decode_routput : (String) -> String raise @base.Fault
  decode_foutput : (String) -> String raise @base.Fault
}

///|
pub fn Model::make(
  self : Model,
  code : String,
  input : String,
) -> Unit raise @base.Fault {
  (self.make)(code, input)
}

///|
pub fn Model::step(self : Model, input : String) -> String raise @base.Fault {
  (self.step)(input)
}

///|
pub fn Model::snapshot(self : Model) -> String raise @base.Fault {
  (self.snapshot)()
}

///|
pub fn Model::restore(self : Model, text : String) -> Unit raise @base.Fault {
  (self.restore)(text)
}

///|
pub fn Model::render(self : Model, text : String) -> String raise @base.Fault {
  (self.render)(text)
}
