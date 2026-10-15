include Syntax

type symbols = {
  consts : (Utils_names.Identifier.t * N.t) list;
  text_labels : (Utils_names.Identifier.t * int) list;
  data_labels : (Utils_names.Identifier.t * int) list;
  code_len : int;
  data_len : int;
}
[@@deriving yojson]

type debug_info = {
  text_labels_by_addr : (int * Utils_names.Identifier.t list) list;
  data_labels_by_addr : (int * Utils_names.Identifier.t list) list;
}
[@@deriving yojson]

type ainput = asm_input
type rinput = unit
type routput = unit
type foutput = asm_output

type state = {
  code : code;
  insts : inst array;
  symbols : symbols;
  debug_info : debug_info;
  regs : N.t array;
  flag : bool;
  pc : int;
  static_data_len : int;
  static_data : N.t list;
  data_memory : N.t array;
  halted : bool;
}
[@@deriving yojson]
