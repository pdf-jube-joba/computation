///|
pub(open) trait Text {
  fn parse(String) -> Self raise @utils.Fault
  fn print(Self) -> String
}

///|
pub(open) trait Snapshot {
  fn encode(Self) -> Json
  fn decode(Json) -> Self raise @utils.Fault
}

///|
pub(open) trait Render {
  fn render(Self) -> Array[Block]
}

///|
pub(all) enum Transition[S, RO, FO] {
  Continue(S, RO)
  Halt(FO)
}

///|
pub struct Model[C, A, RI, RO, FO, S] {
  make : (C, A) -> S raise @utils.Fault
  step : (S, RI) -> Transition[S, RO, FO] raise @utils.Fault
}

///|
pub struct Compiler[SC, SA, SRI, SRO, SFO, TC, TA, TRI, TRO, TFO] {
  compile_code : (SC) -> TC raise @utils.Fault
  encode_ainput : (SA) -> TA raise @utils.Fault
  encode_rinput : (SRI) -> TRI raise @utils.Fault
  decode_routput : (TRO) -> SRO raise @utils.Fault
  decode_foutput : (TFO) -> SFO raise @utils.Fault
}

///|
// These bounds enforce the construction contract, even though construction calls no trait methods.
#warnings("-53")
pub fn[
  C : Text,
  A : Text,
  RI : Text,
  RO : Text,
  FO : Text,
  S : Snapshot + Render,
] Model::Model(
  make~ : (C, A) -> S raise @utils.Fault,
  step~ : (S, RI) -> Transition[S, RO, FO] raise @utils.Fault,
) -> Model[C, A, RI, RO, FO, S] {
  { make, step, }
}

///|
#warnings("-53")
pub fn[
  SC : Text,
  SA : Text,
  SRI : Text,
  SRO : Text,
  SFO : Text,
  TC : Text,
  TA : Text,
  TRI : Text,
  TRO : Text,
  TFO : Text,
] Compiler::Compiler(
  compile_code~ : (SC) -> TC raise @utils.Fault,
  encode_ainput~ : (SA) -> TA raise @utils.Fault,
  encode_rinput~ : (SRI) -> TRI raise @utils.Fault,
  decode_routput~ : (TRO) -> SRO raise @utils.Fault,
  decode_foutput~ : (TFO) -> SFO raise @utils.Fault,
) -> Compiler[SC, SA, SRI, SRO, SFO, TC, TA, TRI, TRO, TFO] {
  {
    compile_code,
    encode_ainput,
    encode_rinput,
    decode_routput,
    decode_foutput,
  }
}

///|
pub fn[
  AC : Text,
  AA : Text,
  ARI : Text,
  ARO : Text,
  AFO : Text,
  BC,
  BA,
  BRI,
  BRO,
  BFO,
  CC : Text,
  CA : Text,
  CRI : Text,
  CRO : Text,
  CFO : Text,
] compose(
  first : Compiler[AC, AA, ARI, ARO, AFO, BC, BA, BRI, BRO, BFO],
  second : Compiler[BC, BA, BRI, BRO, BFO, CC, CA, CRI, CRO, CFO],
) -> Compiler[AC, AA, ARI, ARO, AFO, CC, CA, CRI, CRO, CFO] {
  Compiler(
    compile_code=code => (second.compile_code)((first.compile_code)(code)),
    encode_ainput=input => (second.encode_ainput)((first.encode_ainput)(input)),
    encode_rinput=input => (second.encode_rinput)((first.encode_rinput)(input)),
    decode_routput=output => {
      (first.decode_routput)((second.decode_routput)(output))
    },
    decode_foutput=output => {
      (first.decode_foutput)((second.decode_foutput)(output))
    },
  )
}

///|
pub impl Text for Unit with fn parse(text) {
  @utils.unit_input(text)
}

///|
pub impl Text for Unit with fn print(_) {
  ""
}

///|
pub impl Text for String with fn parse(text) {
  text
}

///|
pub impl Text for String with fn print(text) {
  text
}

///|
pub impl Text for UInt32 with fn parse(text) {
  @utils.unsigned(text)
}

///|
pub impl Text for UInt32 with fn print(value) {
  value.to_string()
}

///|
pub impl Text for @utils.Nat with fn parse(text) {
  @utils.nat(text)
}

///|
pub impl Text for @utils.Nat with fn print(value) {
  value.to_string()
}
