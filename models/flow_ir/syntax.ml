module N = Utils_base.Number

type binop = Add | Sub [@@deriving yojson]

type value_expr =
  | VReg of Utils_names.Identifier.t
  | Imm of N.t
  | CodeLabel of Utils_names.Identifier.t
  | Ref of place_expr

and place_expr =
  | Label of Utils_names.Identifier.t
  | Deref of Utils_names.Identifier.t
  | SAcc of value_expr
  | HAcc of value_expr * value_expr
[@@deriving yojson]

type cond =
  | Lt of value_expr * value_expr
  | Eq of value_expr * value_expr
  | Gt of value_expr * value_expr
[@@deriving yojson]

type stmt =
  | Nop
  | Load of Utils_names.Identifier.t * place_expr
  | Assign of Utils_names.Identifier.t * value_expr
  | BinOp of Utils_names.Identifier.t * value_expr * binop * value_expr
  | Store of place_expr * value_expr
  | Pop of Utils_names.Identifier.t
  | Push of value_expr
  | LGet of Utils_names.Identifier.t
  | HAlloc of value_expr * Utils_names.Identifier.t
  | HFree of value_expr
  | Print of Utils_names.Identifier.t
  | Input of place_expr
[@@deriving yojson]

type jump_if = { cond : cond; target : value_expr } [@@deriving yojson]

type cont =
  | Go of jump_if list * value_expr
  | Enter of jump_if list * value_expr
  | Halt of jump_if list
[@@deriving yojson]

type block = {
  label : Utils_names.Identifier.t;
  stmts : stmt list;
  cont : cont;
}
[@@deriving yojson]

type region = { label : Utils_names.Identifier.t; blocks : block list }
[@@deriving yojson]

type static_def = { label : Utils_names.Identifier.t; value : N.t }
[@@deriving yojson]

type code = { statics : static_def list; regions : region list }
[@@deriving yojson]

let ifs = function Go (xs, _) | Enter (xs, _) | Halt xs -> xs
