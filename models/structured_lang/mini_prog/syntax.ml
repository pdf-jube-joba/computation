module N = Utils_base.Number

type ty =
  | Num
  | U8
  | Bool
  | Unit
  | Ptr
  | Fn
  | Product of ty list
  | Sum of ty list
  | Array of ty * int
[@@deriving yojson]

type binop = Add | Sub | Eq | Lt | And | Or [@@deriving yojson]
type unop = Not [@@deriving yojson]

type place_expr =
  | Static of Utils_names.Identifier.t
  | Local of Utils_names.Identifier.t
  | Field of place_expr * int
  | Tag_place of place_expr * int
  | Index of place_expr * value_expr
  | Deref of value_expr

and value_expr =
  | Number of N.t
  | Char of string
  | Boolean of bool
  | Unit_value
  | NullPtr
  | NullFn
  | BinOp of value_expr * binop * value_expr
  | UnOp of unop * value_expr
  | Pair of value_expr list
  | Tag of int * value_expr
  | Load of place_expr
  | Addr of place_expr
  | Function of Utils_names.Identifier.t
[@@deriving yojson]

type stmt =
  | Assign of place_expr * value_expr
  | If of value_expr * stmt
  | Case of int * value_expr * stmt
  | HAlloc of ty * place_expr
  | HFree of value_expr
  | Loop of Utils_names.Identifier.t * stmt
  | Break of Utils_names.Identifier.t
  | Continue of Utils_names.Identifier.t
  | Call of value_expr * value_expr list * place_expr list
  | Return of value_expr list
  | Block of block

and block = {
  bindings : (Utils_names.Identifier.t * ty) list;
  stmts : stmt list;
}
[@@deriving yojson]

type static_decl = { name : Utils_names.Identifier.t; ty : ty }
[@@deriving yojson]

type fn_decl = { name : Utils_names.Identifier.t; block : block }
[@@deriving yojson]

type code = { statics : static_decl list; functions : fn_decl list }
[@@deriving yojson]
