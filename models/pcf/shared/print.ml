open Syntax

let print_binop = function Add -> "+" | Sub -> "-" | And -> "&&"
let print_unop = function Inc -> "inc" | Dec -> "dec" | Not -> "not"

let rec print = function
  | Nat n -> N.to_string n
  | Bool true -> "#true"
  | Bool false -> "#false"
  | Unit -> "#unit"
  | Print n -> "print " ^ N.to_string n
  | Var x -> Utils_names.Identifier.to_string x
  | BinOp (a, o, b) -> "(" ^ print a ^ " " ^ print_binop o ^ " " ^ print b ^ ")"
  | UnOp (o, e) -> "(" ^ print_unop o ^ " " ^ print e ^ ")"
  | Fun (x, b) ->
      "(fun " ^ Utils_names.Identifier.to_string x ^ " => " ^ print b ^ ")"
  | Rec (f, x, b) ->
      "(rec "
      ^ Utils_names.Identifier.to_string f
      ^ " "
      ^ Utils_names.Identifier.to_string x
      ^ " => " ^ print b ^ ")"
  | App (f, x) -> print f ^ "(" ^ print x ^ ")"
  | If (c, t, e) ->
      "(if " ^ print c ^ " then " ^ print t ^ " else " ^ print e ^ " fi)"
  | Block (s, e) -> "{ " ^ print_stmt s ^ "; " ^ print e ^ " }"

and print_stmt = function
  | Skip -> "skip"
  | PrintStmt n -> "print " ^ N.to_string n
  | Seq (a, b) -> print_stmt a ^ "; " ^ print_stmt b
  | Let (x, e) -> "let " ^ Utils_names.Identifier.to_string x ^ " := " ^ print e
  | IfStmt (c, s) -> "if " ^ print c ^ " then " ^ print_stmt s ^ " end"
  | While (c, s) -> "while " ^ print c ^ " do " ^ print_stmt s ^ " end"
