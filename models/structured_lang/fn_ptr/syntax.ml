type binop = Add | Sub [@@deriving yojson]

type place_expr = Var of Utils_names.Identifier.t | Deref of value_expr

and value_expr =
  | Number of int64
  | NullPtr
  | BinOp of value_expr * binop * value_expr
  | Load of place_expr
  | Addr of place_expr
[@@deriving yojson]

type stmt =
  | Assign of place_expr * value_expr
  | Ifz of value_expr * stmt * stmt
  | Call of Utils_names.Identifier.t * value_expr list
  | Return
[@@deriving yojson]

type fn_decl = {
  name : Utils_names.Identifier.t;
  params : Utils_names.Identifier.t list;
  body : stmt list;
}
[@@deriving yojson]

type code = { functions : fn_decl list } [@@deriving yojson]
