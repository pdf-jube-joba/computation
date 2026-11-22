///|
pub struct Session {
  make : (String, String) -> Unit raise @utils.Fault
  step : (String) -> String raise @utils.Fault
  snapshot : () -> String raise @utils.Fault
  restore : (String) -> Unit raise @utils.Fault
  render : (String) -> String raise @utils.Fault
}

///|
pub fn[
  C : @definition.Text,
  A : @definition.Text,
  RI : @definition.Text,
  RO : @definition.Text,
  FO : @definition.Text,
  S : @definition.Snapshot + @definition.Render,
] session(
  model : @definition.Model[C, A, RI, RO, FO, S],
) -> Session {
  let state : Ref[S?] = { val: None, }
  fn current() -> S raise @utils.Fault {
    match state.val {
      Some(value) => value
      None => @utils.error("No active machine; call make first")
    }
  }
  {
    make: (code, input) => {
      let value = (model.make)(
        @definition.Text::parse(code),
        @definition.Text::parse(input),
      )
      state.val = Some(value)
    },
    step: input => {
      match (model.step)(current(), @definition.Text::parse(input)) {
        @definition.Continue(next, output) => {
          let response = @utils.object([
            ("kind", @utils.jstr("continue")),
            ("routput", @utils.jstr(@definition.Text::print(output))),
          ]).stringify()
          state.val = Some(next)
          response
        }
        @definition.Halt(output) => {
          let response = @utils.object([
            ("kind", @utils.jstr("halt")),
            ("foutput", @utils.jstr(@definition.Text::print(output))),
          ]).stringify()
          state.val = None
          response
        }
      }
    },
    snapshot: () => @definition.Snapshot::encode(current()).stringify(),
    restore: text => {
      let value = @definition.Snapshot::decode(@utils.parse_json(text))
      state.val = Some(value)
    },
    render: text => {
      let value : S = @definition.Snapshot::decode(@utils.parse_json(text))
      render_json(@definition.Render::render(value)).stringify()
    },
  }
}

///|
pub(all) struct CompilerSession {
  compile_code : (String) -> String raise @utils.Fault
  encode_ainput : (String) -> String raise @utils.Fault
  encode_rinput : (String) -> String raise @utils.Fault
  decode_routput : (String) -> String raise @utils.Fault
  decode_foutput : (String) -> String raise @utils.Fault
}

///|
pub fn[
  SC : @definition.Text,
  SA : @definition.Text,
  SRI : @definition.Text,
  SRO : @definition.Text,
  SFO : @definition.Text,
  TC : @definition.Text,
  TA : @definition.Text,
  TRI : @definition.Text,
  TRO : @definition.Text,
  TFO : @definition.Text,
] compiler_session(
  compiler : @definition.Compiler[SC, SA, SRI, SRO, SFO, TC, TA, TRI, TRO, TFO],
) -> CompilerSession {
  {
    compile_code: text => {
      @definition.Text::print(
        (compiler.compile_code)(@definition.Text::parse(text)),
      )
    },
    encode_ainput: text => {
      @definition.Text::print(
        (compiler.encode_ainput)(@definition.Text::parse(text)),
      )
    },
    encode_rinput: text => {
      @definition.Text::print(
        (compiler.encode_rinput)(@definition.Text::parse(text)),
      )
    },
    decode_routput: text => {
      @definition.Text::print(
        (compiler.decode_routput)(@definition.Text::parse(text)),
      )
    },
    decode_foutput: text => {
      @definition.Text::print(
        (compiler.decode_foutput)(@definition.Text::parse(text)),
      )
    },
  }
}

///|
pub fn Session::make(
  self : Session,
  code : String,
  input : String,
) -> Unit raise @utils.Fault {
  (self.make)(code, input)
}

///|
pub fn Session::step(
  self : Session,
  input : String,
) -> String raise @utils.Fault {
  (self.step)(input)
}

///|
pub fn Session::snapshot(self : Session) -> String raise @utils.Fault {
  (self.snapshot)()
}

///|
pub fn Session::restore(
  self : Session,
  text : String,
) -> Unit raise @utils.Fault {
  (self.restore)(text)
}

///|
pub fn Session::render(
  self : Session,
  text : String,
) -> String raise @utils.Fault {
  (self.render)(text)
}
