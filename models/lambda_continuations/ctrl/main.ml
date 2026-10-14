include Lambda_continuations_shared.Syntax

let unsupported =
  "abort/control semantics are not implemented yet (unsupported in the \
   original Rust model)"

type code = term
type ainput = Lambda_continuations_no_ext.Main.ainput
type rinput = unit
type routput = unit
type foutput = value
type state = Lambda_continuations_no_ext.Main.state [@@deriving yojson]

module Code = Lambda_continuations_shared.Syntax.Ctrl_code

let make = Lambda_continuations_no_ext.Main.make
let step _ () = Error unsupported
let snapshot = Lambda_continuations_no_ext.Main.snapshot
let restore = Lambda_continuations_no_ext.Main.restore
let render = Lambda_continuations_no_ext.Main.render

module Ainput = Utils_codec.Text.Natural_list
module Rinput = Utils_codec.Text.Unit
module Routput = Utils_codec.Text.Unit
module Foutput = Lambda_continuations_shared.Syntax.Value
