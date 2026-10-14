include module type of struct
  include Types
end

include
  Utils_runtime.Machine.S
    with type state := state
     and type Code.t = code
     and type Ainput.t = ainput
     and type Rinput.t = rinput
     and type Routput.t = routput
     and type Foutput.t = foutput
