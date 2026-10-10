open Common

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
  | Static of string
  | Local of string
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
  | Function of string
[@@deriving yojson]

type stmt =
  | Assign of place_expr * value_expr
  | If of value_expr * stmt
  | Case of int * value_expr * stmt
  | HAlloc of ty * place_expr
  | HFree of value_expr
  | Loop of string * stmt
  | Break of string
  | Continue of string
  | Call of value_expr * value_expr list * place_expr list
  | Return of value_expr list
  | Block of block

and block = { bindings : (string * ty) list; stmts : stmt list }
[@@deriving yojson]

type static_decl = { name : string; ty : ty } [@@deriving yojson]
type fn_decl = { name : string; block : block } [@@deriving yojson]

type code = { statics : static_decl list; functions : fn_decl list }
[@@deriving yojson]

type location_key = Heap of int | Static_key of string | Local_key of int
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
  | FnPtr of string option
  | Aggregate of value list
  | Sum_value of int * value
[@@deriving yojson]

type cell = { ty : ty; value : value } [@@deriving yojson]

type store = {
  heap : (int * cell) list;
  statics : (string * cell) list;
  locals : (int * cell) list list list;
}
[@@deriving yojson]

type control = Next | Break_ctrl of string | Continue_ctrl of string
[@@deriving yojson]

type exec = Eval of stmt | Ctrl of control | Ret of value list
[@@deriving yojson]

type inner_frame = Stmt_frame of stmt | Scope | Loop_frame of string * stmt
[@@deriving yojson]

type fn_frame = {
  function_name : string;
  env : (string * int) list list;
  ret_locs : location list;
  inner_k : inner_frame list;
}
[@@deriving yojson]

type state = {
  code : code;
  current_function : string;
  exec : exec;
  env : (string * int) list list;
  store : store;
  inner_k : inner_frame list;
  fn_k : fn_frame list;
  next_local_id : int;
  next_heap_id : int;
}
[@@deriving yojson]

type ainput = N.t list [@@deriving yojson]
type rinput = unit
type routput = string
type foutput = N.t [@@deriving yojson]

let rec type_text = function
  | Num -> "#num"
  | U8 -> "#u8"
  | Bool -> "#bool"
  | Unit -> "#unit"
  | Ptr -> "#ptr"
  | Fn -> "#fn"
  | Product ts -> String.concat " * " (List.map type_text ts)
  | Sum ts -> String.concat " + " (List.map type_text ts)
  | Array (t, n) -> "[" ^ type_text t ^ "; " ^ string_of_int n ^ "]"

let op_text = function
  | Add -> "+"
  | Sub -> "-"
  | Eq -> "=="
  | Lt -> "<"
  | And -> "&&"
  | Or -> "||"

let rec place_text = function
  | Static n -> "static " ^ n
  | Local n -> "local " ^ n
  | Field (p, i) -> place_text p ^ "." ^ string_of_int i
  | Tag_place (p, i) -> place_text p ^ "?" ^ string_of_int i
  | Index (p, v) -> place_text p ^ "[" ^ expr_text v ^ "]"
  | Deref v -> "(" ^ expr_text v ^ ") #loc"

and expr_text = function
  | Number n -> N.to_string n
  | Char c -> "'" ^ c ^ "'"
  | Boolean b -> if b then "#true" else "#false"
  | Unit_value -> "#unit"
  | NullPtr -> "#null-ptr"
  | NullFn -> "#null-fn"
  | BinOp (a, op, b) -> expr_text a ^ " " ^ expr_text b ^ " " ^ op_text op
  | UnOp (Not, v) -> expr_text v ^ " !"
  | Pair vs ->
      String.concat ", " (List.map expr_text vs)
      ^ " pair("
      ^ string_of_int (List.length vs)
      ^ ")"
  | Tag (t, v) -> expr_text v ^ " tag(" ^ string_of_int t ^ ")"
  | Load p -> "ld " ^ place_text p
  | Addr p -> place_text p ^ " #addr"
  | Function n -> "fn " ^ n

let rec stmt_text = function
  | Assign (p, v) -> "assign " ^ place_text p ^ " := " ^ expr_text v
  | If (c, s) -> "if " ^ expr_text c ^ " " ^ stmt_text s
  | Case (t, v, s) ->
      "case " ^ string_of_int t ^ " of " ^ expr_text v ^ " " ^ stmt_text s
  | HAlloc (t, p) -> "halloc " ^ type_text t ^ " -> " ^ place_text p
  | HFree v -> "hfree " ^ expr_text v
  | Loop (l, s) -> "loop " ^ l ^ ": " ^ stmt_text s
  | Break l -> "break " ^ l
  | Continue l -> "continue " ^ l
  | Call (f, args, rets) ->
      "call " ^ expr_text f ^ "("
      ^ String.concat ", " (List.map expr_text args)
      ^ ") -> "
      ^ String.concat ", " (List.map place_text rets)
  | Return vs ->
      "return"
      ^ if vs = [] then "" else " " ^ String.concat ", " (List.map expr_text vs)
  | Block b -> "block " ^ block_text b

and block_text b =
  "("
  ^ String.concat ", "
      (List.map (fun (n, t) -> n ^ ": " ^ type_text t) b.bindings)
  ^ ") { "
  ^ String.concat "; " (List.map stmt_text b.stmts)
  ^ " }"

let print (c : code) =
  String.concat "\n"
    (List.map
       (fun (s : static_decl) -> "static " ^ s.name ^ ": " ^ type_text s.ty)
       c.statics
    @ List.map
        (fun (f : fn_decl) -> "fn " ^ f.name ^ block_text f.block)
        c.functions)

let location_text loc =
  (match loc.key with
    | Heap n -> "heap#" ^ string_of_int n
    | Static_key n -> "static " ^ n
    | Local_key n -> "local#" ^ string_of_int n)
  ^ String.concat ""
      (List.map
         (function
           | Field_proj n -> "." ^ string_of_int n
           | Tag_proj n -> "?" ^ string_of_int n
           | Index_proj n -> "[" ^ string_of_int n ^ "]")
         loc.projections)

let rec value_text = function
  | Number_value n -> N.to_string n
  | Char_value c -> "'" ^ c ^ "'"
  | Bool_value b -> if b then "#true" else "#false"
  | Unit_v -> "#unit"
  | Ptr_value None -> "#null-ptr"
  | Ptr_value (Some l) -> location_text l
  | FnPtr None -> "#null-fn"
  | FnPtr (Some n) -> "fn " ^ n
  | Aggregate vs -> "(" ^ String.concat ", " (List.map value_text vs) ^ ")"
  | Sum_value (t, v) -> "tag(" ^ string_of_int t ^ ", " ^ value_text v ^ ")"

let exec_text = function
  | Eval s -> "eval " ^ stmt_text s
  | Ctrl Next -> "ctrl next"
  | Ctrl (Break_ctrl l) -> "ctrl break " ^ l
  | Ctrl (Continue_ctrl l) -> "ctrl continue " ^ l
  | Ret vs -> "ret " ^ String.concat ", " (List.map value_text vs)

let inner_text = function
  | Stmt_frame s -> "stmt " ^ stmt_text s
  | Scope -> "scope"
  | Loop_frame (l, s) -> "loop " ^ l ^ ": " ^ stmt_text s
