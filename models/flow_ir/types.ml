include Syntax
module Vreg_id = Utils_names.Id.Make ()
module Static_id = Utils_names.Id.Make ()
module Region_id = Utils_names.Id.Make ()
module Block_id = Utils_names.Id.Make ()
module Heap_id = Utils_names.Id.Make ()

type code_target = Region_target of Region_id.t | Block_target of Block_id.t
[@@deriving yojson]

type 'data location = Static of 'data | Stack of int | Heap of Heap_id.t * int
[@@deriving yojson]

type ('label, 'data) value =
  | Num of N.t
  | Code_label of 'label
  | Key of 'data location
[@@deriving yojson]

type place_key = Static_id.t location [@@deriving yojson]

type flow_value = (Utils_names.Identifier.t, Utils_names.Identifier.t) value
[@@deriving yojson]

type runtime_value = (code_target, Static_id.t) value [@@deriving yojson]

type resolution = {
  registers : (Utils_names.Identifier.t * Vreg_id.t) list;
  data : (Utils_names.Identifier.t * Static_id.t) list;
  regions : (Utils_names.Identifier.t * Region_id.t) list;
  blocks : (Utils_names.Identifier.t * Block_id.t) list list;
  label_names : (code_target * Utils_names.Identifier.t) list;
}
[@@deriving yojson]

type ainput = (Utils_names.Identifier.t * flow_value) list [@@deriving yojson]
type foutput = ainput [@@deriving yojson]
type rinput = string
type routput = string

type state = {
  code : code;
  current_region : int;
  current_block : int;
  current_line : int;
  resolution : resolution;
  vregs : (Vreg_id.t * runtime_value) list;
  static_mem : (Static_id.t * runtime_value) list;
  stack : runtime_value list;
  heap : (Heap_id.t * runtime_value list) list;
  next_handle : Heap_id.Supply.t;
  halted : bool;
}
[@@deriving yojson]
