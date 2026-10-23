type ('state, 'routput, 'foutput) step_result =
  | Continue of { next : 'state; output : 'routput }
  | Halt of 'foutput

module type S = sig
  type state

  include Language.S

  val make : Code.t -> Ainput.t -> (state, string) result

  val step :
    state ->
    Rinput.t ->
    ((state, Routput.t, Foutput.t) step_result, string) result

  module Snapshot : Snapshot.S with type t = state

  val render : state -> Render.t
end

let ( let* ) = Result.bind
