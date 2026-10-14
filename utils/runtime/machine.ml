type ('state, 'routput, 'foutput) step_result =
  | Continue of { next : 'state; output : 'routput }
  | Halt of 'foutput

module type TEXT = Utils_codec.Text.S

module type S = sig
  type state

  include Language.S

  val make : Code.t -> Ainput.t -> (state, string) result

  val step :
    state ->
    Rinput.t ->
    ((state, Routput.t, Foutput.t) step_result, string) result

  val snapshot : state -> Yojson.Safe.t
  val restore : Yojson.Safe.t -> (state, string) result
  val render : state -> Yojson.Safe.t
end

let ( let* ) = Result.bind
