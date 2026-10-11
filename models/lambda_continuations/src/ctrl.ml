include Lambda

let unsupported =
  "abort/control semantics are not implemented yet (unsupported in the \
   original Rust model)"

type code = term
type ainput = No_ext.ainput
type rinput = unit
type routput = unit
type foutput = value
type state = No_ext.state [@@deriving yojson]

module Code = Lambda.Ctrl_code

let make = No_ext.make
let step _ () = Error unsupported
let snapshot = No_ext.snapshot
let restore = No_ext.restore
let render = No_ext.render

module Ainput = Utils.Text.Natural_list
module Rinput = Utils.Text.Unit
module Routput = Utils.Text.Unit
module Foutput = Lambda.Value
