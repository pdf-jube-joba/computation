open Structured_lang_shared.Arithmetic
open Syntax

type t = code

let rec place_to_text = function
  | Var v -> Utils_names.Identifier.to_string v
  | Deref v -> "(" ^ value_to_text v ^ ") \\loc"

and value_to_text = function
  | Number n -> Int64.to_string n
  | NullPtr -> "\\null_ptr"
  | BinOp (a, op, b) ->
      "(" ^ value_to_text a
      ^ (if op = Add then " + " else " - ")
      ^ value_to_text b ^ ")"
  | Load p -> "\\ld " ^ place_to_text p
  | Addr p -> place_to_text p ^ " \\addr"

let rec stmt_to_text = function
  | Assign (p, v) -> "\\assign " ^ place_to_text p ^ " := " ^ value_to_text v
  | Ifz (c, a, b) ->
      "\\ifz " ^ value_to_text c ^ " \\then " ^ stmt_to_text a ^ " \\else "
      ^ stmt_to_text b ^ " \\end"
  | Call (n, args) ->
      "\\call "
      ^ Utils_names.Identifier.to_string n
      ^ "("
      ^ String.concat ", " (List.map value_to_text args)
      ^ ")"
  | Return -> "\\return"

let print c =
  String.concat "\n\n"
    (List.map
       (fun f ->
         "\\fn "
         ^ Utils_names.Identifier.to_string f.name
         ^ "("
         ^ String.concat ", "
             (List.map Utils_names.Identifier.to_string f.params)
         ^ ") { "
         ^ String.concat "; " (List.map stmt_to_text f.body)
         ^ " }")
       c.functions)

let rec parse_place p =
  match L.peek p with
  | L.Ident _ -> Var (L.identifier p)
  | _ ->
      let v = L.group p L.Parentheses parse_value in
      L.expect p "\\loc";
      Deref v

and parse_value p =
  L.expression
    ~infix:
      [
        ("+", 1, 2, fun a b -> BinOp (a, Add, b));
        ("-", 1, 2, fun a b -> BinOp (a, Sub, b));
      ]
    ~atom:(fun _ p -> parse_atom p)
    p

and parse_atom p =
  if L.bump_if p "\\null_ptr" then NullPtr
  else if L.bump_if_keyword p "ld" then Load (parse_place p)
  else if L.is_group p L.Parentheses then L.group p L.Parentheses parse_value
  else
    match L.peek p with
    | L.Number _ -> Number (Int64.of_string (N.to_string (L.natural p)))
    | _ ->
        let v = parse_place p in
        L.expect p "\\addr";
        Addr v

let rec parse_stmt p =
  if L.bump_if_keyword p "assign" then (
    let place = parse_place p in
    L.expect p ":=";
    Assign (place, parse_value p))
  else if L.bump_if_keyword p "ifz" then (
    let cond = parse_value p in
    L.expect_keyword p "then";
    let a = parse_stmt p in
    L.expect_keyword p "else";
    let b = parse_stmt p in
    L.expect_keyword p "end";
    Ifz (cond, a, b))
  else if L.bump_if_keyword p "call" then
    let n = L.identifier p in
    let args =
      L.group p L.Parentheses (fun p ->
          L.separated_list p ~separator:',' parse_value)
    in
    Call (n, args)
  else (
    L.expect_keyword p "return";
    Return)

let parse =
  L.parse (fun p ->
      let functions =
        L.many p (fun p ->
            L.expect_keyword p "fn";
            let name = L.identifier p in
            let params =
              L.group p L.Parentheses (fun p ->
                  L.separated_list p ~separator:',' L.identifier)
            in
            let body =
              L.group p L.Braces (fun p ->
                  L.separated_list ~allow_trailing:true p ~separator:';'
                    parse_stmt)
            in
            { name; params; body })
      in
      { functions })
