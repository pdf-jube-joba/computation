include Syntax
include Identity

type location = { frame_id : Frame_id.t; slot : int } [@@deriving yojson]

type value = Num of int64 | Location of location | Null_ptr
[@@deriving yojson]

type frame_store = { frame_id : Frame_id.t; values : (location * value) list }
[@@deriving yojson]

type call_frame = {
  env : (Utils_names.Identifier.t * location) list;
  inner_k : stmt list;
  function_name : Utils_names.Identifier.t;
}
[@@deriving yojson]

type state = {
  code : code;
  current_function : Utils_names.Identifier.t;
  current_stmt : stmt option;
  env : (Utils_names.Identifier.t * location) list;
  store : frame_store list;
  inner_k : stmt list;
  fn_k : call_frame list;
  next_frame_id : Frame_id.Supply.t;
}
[@@deriving yojson]

type ainput = unit
type rinput = unit
type routput = string
type foutput = string
