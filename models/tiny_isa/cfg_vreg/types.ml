include Syntax

type compiled_program = {
  blocks : block array;
  entry_block : int;
  block_index : (Utils_names.Identifier.t * int) list;
  static_labels : (Utils_names.Identifier.t * int) list;
}
[@@deriving yojson]

type ainput = N.t list
type rinput = unit
type routput = unit
type foutput = N.t list

type state = {
  code : code;
  compiled : compiled_program;
  current_block : int;
  vregs : N.t array;
  memory : N.t array;
  halted : bool;
}
[@@deriving yojson]
