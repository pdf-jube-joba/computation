module N = Utils_base.Number
module Id = Utils_names.Identifier

type value_expr = Reg of Id.t | Imm of N.t | Label of Id.t | Memory of int
[@@deriving yojson]

type place = Deref of value_expr [@@deriving yojson]
type binop = Add | Sub [@@deriving yojson]

type rhs =
  | Value of value_expr
  | Binop of value_expr * binop * value_expr
  | Load of place
  | Slice of value_expr * value_expr * value_expr
  | Addr of value_expr * value_expr
  | Len of value_expr
[@@deriving yojson]

type stmt = Assign of Id.t * rhs | Store of place * value_expr
[@@deriving yojson]

type rel = Eq | Lt | Gt [@@deriving yojson]
type cond = value_expr * rel * value_expr [@@deriving yojson]
type jump_if = { cond : cond; target : value_expr } [@@deriving yojson]
type ending = Goto of value_expr | Halt [@@deriving yojson]

type block = {
  label : Id.t;
  stmts : stmt list;
  ifs : jump_if list;
  ending : ending;
}
[@@deriving yojson]

type code = { memories : int; blocks : block list } [@@deriving yojson]
