include Syntax
module Local_id = Utils_names.Id.Make ()
module Heap_id = Utils_names.Id.Make ()

type location_key =
  | Heap of Heap_id.t
  | Static_key of Utils_names.Identifier.t
  | Local_key of Local_id.t
[@@deriving yojson]

type projection = Field_proj of int | Tag_proj of int | Index_proj of int
[@@deriving yojson]

type location = { key : location_key; projections : projection list }
[@@deriving yojson]

type value =
  | Number_value of N.t
  | Char_value of string
  | Bool_value of bool
  | Unit_v
  | Ptr_value of location option
  | FnPtr of Utils_names.Identifier.t option
  | Aggregate of value list
  | Sum_value of int * value
[@@deriving yojson]

type cell = { ty : ty; value : value } [@@deriving yojson]

type store = {
  heap : (Heap_id.t * cell) list;
  statics : (Utils_names.Identifier.t * cell) list;
  locals : (Local_id.t * cell) list list list;
}
[@@deriving yojson]

type control =
  | Next
  | Break_ctrl of Utils_names.Identifier.t
  | Continue_ctrl of Utils_names.Identifier.t
[@@deriving yojson]

type exec = Eval of stmt | Ctrl of control | Ret of value list
[@@deriving yojson]

type inner_frame =
  | Stmt_frame of stmt
  | Scope
  | Loop_frame of Utils_names.Identifier.t * stmt
[@@deriving yojson]

type fn_frame = {
  function_name : Utils_names.Identifier.t;
  env : (Utils_names.Identifier.t * Local_id.t) list list;
  ret_locs : location list;
  inner_k : inner_frame list;
}
[@@deriving yojson]

type state = {
  code : code;
  current_function : Utils_names.Identifier.t;
  exec : exec;
  env : (Utils_names.Identifier.t * Local_id.t) list list;
  store : store;
  inner_k : inner_frame list;
  fn_k : fn_frame list;
  next_local_id : Local_id.Supply.t;
  next_heap_id : Heap_id.Supply.t;
}
[@@deriving yojson]

type ainput = N.t list [@@deriving yojson]
type rinput = unit
type routput = string
type foutput = N.t [@@deriving yojson]
