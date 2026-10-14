type code = Pcf_shared.Syntax.expr
type ainput = Utils_base.Number.t list
type rinput = unit
type routput = Pcf_shared.Syntax.print_effect
type foutput = Utils_base.Number.t

include
  Utils_runtime.Machine.S
    with type Code.t = code
     and type Ainput.t = ainput
     and type Rinput.t = rinput
     and type Routput.t = routput
     and type Foutput.t = foutput
