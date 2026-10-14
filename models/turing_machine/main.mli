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

include
  Utils_runtime.Machine.S
    with type state := state
     and module Code := Code
     and module Ainput := Ainput
     and module Rinput := Rinput
     and module Routput := Routput
     and module Foutput := Foutput
