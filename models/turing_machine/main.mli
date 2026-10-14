include module type of struct
  include Types
end

module Sign = Sign
module State = State
module Direction = Direction
module Tape = Tape
module Code = Code
module Ainput = Tape
module Rinput = Utils_codec.Text.Unit
module Routput = Utils_codec.Text.Unit
module Foutput = Tape

val unwrap : ('a, string) result -> 'a
val make : code -> tape -> (state, string) result

val step :
  state ->
  unit ->
  ((state, unit, tape) Utils_runtime.Machine.step_result, string) result

val snapshot : state -> Yojson.Safe.t
val restore : Yojson.Safe.t -> (state, string) result
val render : state -> Yojson.Safe.t
