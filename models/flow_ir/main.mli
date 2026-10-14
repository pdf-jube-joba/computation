include module type of struct
  include Types
end

module Code : Utils_codec.Text.S with type t = code

module Ainput : sig
  include Utils_codec.Text.S with type t = ainput

  val print_value : flow_value -> string
end

module Rinput : Utils_codec.Text.S with type t = rinput
module Routput : Utils_codec.Text.S with type t = routput
module Foutput : Utils_codec.Text.S with type t = foutput

val make : code -> ainput -> (state, string) result

val step :
  state ->
  rinput ->
  ((state, routput, foutput) Utils_runtime.Machine.step_result, string) result

val snapshot : state -> Yojson.Safe.t
val restore : Yojson.Safe.t -> (state, string) result
val render : state -> Yojson.Safe.t
