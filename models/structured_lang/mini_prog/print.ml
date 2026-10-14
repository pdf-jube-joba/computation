open Syntax

type t = code

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
  | NullPtr -> "#null_ptr"
  | NullFn -> "#null_fn"
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
