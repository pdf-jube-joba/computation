type ('state, 'routput, 'foutput) step_result =
  | Continue of { next : 'state; output : 'routput }
  | Halt of 'foutput

module type TEXT = Text.S

module type S = sig
  type code
  type state

  module Ainput : TEXT
  module Rinput : TEXT
  module Routput : TEXT
  module Foutput : TEXT

  val parse_code : string -> (code, string) result
  val make : code -> Ainput.t -> (state, string) result

  val step :
    state ->
    Rinput.t ->
    ((state, Routput.t, Foutput.t) step_result, string) result

  val snapshot : state -> Yojson.Safe.t
  val restore : Yojson.Safe.t -> (state, string) result
  val render : state -> Yojson.Safe.t
end

let ( let* ) = Result.bind

let text_render text =
  `List [ `Assoc [ ("kind", `String "text"); ("text", `String text) ] ]
