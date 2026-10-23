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

module Snapshot = Lambda_continuations_no_ext.Main.Snapshot

let render = Lambda_continuations_no_ext.Main.render

module Ainput = Utils_data.Text.Natural_list
module Rinput = Utils_data.Text.Unit
module Routput = Utils_data.Text.Unit
module Foutput = Lambda_continuations_shared.Syntax.Value
