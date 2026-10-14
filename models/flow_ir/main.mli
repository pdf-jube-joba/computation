include module type of struct
  include Types
end

module Ainput : sig
  include Utils_codec.Text.S with type t = ainput

  val print_value : flow_value -> string
end

include
  Utils_runtime.Machine.S
    with type state := state
     and type Code.t = code
     and module Ainput := Ainput
     and type Rinput.t = rinput
     and type Routput.t = routput
     and type Foutput.t = foutput
