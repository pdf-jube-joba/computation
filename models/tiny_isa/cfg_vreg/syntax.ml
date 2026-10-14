module N = Utils_base.Number
module Text = Utils_base.String_ext
open Tiny_isa_shared.Instruction

type addr_expr = VReg of int | Label of Utils_names.Identifier.t | Imm of N.t
[@@deriving yojson]

type place_expr = addr_expr [@@deriving yojson]

type value_expr =
  | VRegValue of int
  | ImmValue of N.t
  | LabelValue of Utils_names.Identifier.t
  | Deref of place_expr
[@@deriving yojson]

type binop = Add | Sub [@@deriving yojson]

type cond =
  | Lt of value_expr * value_expr
  | Eq of value_expr * value_expr
  | Gt of value_expr * value_expr
[@@deriving yojson]

type stmt =
  | Assign of int * value_expr
  | BinOp of int * value_expr * binop * value_expr
  | Store of place_expr * value_expr
[@@deriving yojson]

type jump_if = { cond : cond; target : addr_expr } [@@deriving yojson]
type cont = { ifs : jump_if list; jump : addr_expr } [@@deriving yojson]

type block = {
  label : Utils_names.Identifier.t;
  stmts : stmt list;
  cont : cont;
}
[@@deriving yojson]

type static_def = { label : Utils_names.Identifier.t; value : N.t }
[@@deriving yojson]

type code = { statics : static_def list; blocks : block list }
[@@deriving yojson]
