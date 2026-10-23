include module type of struct
  include Types
end

module Code = Code
module Ainput : Utils_data.Text.S with type t = tape list
module Rinput = Utils_data.Text.Unit
module Routput = Utils_data.Text.Unit
module Foutput = Ainput

include
  Utils_runtime.Machine.S
    with type state := state
     and module Code := Code
     and module Ainput := Ainput
     and module Rinput := Rinput
     and module Routput := Routput
     and module Foutput := Foutput
