open Syntax
module L = Utils_parsing.Parser

type t = code

let rec aexp_to_text = function
  | Atom (Var n) -> n
  | Atom (Imm n) -> N.to_string n
  | Bin (a, op, b) ->
      "(" ^ aexp_to_text a ^ " "
      ^ (match op with Add -> "+" | Sub -> "-" | Mul -> "*")
      ^ " " ^ aexp_to_text b ^ ")"

let rec bexp_to_text = function
  | Not b -> "!" ^ bexp_to_text b
  | Rel (a, op, b) ->
      aexp_to_text a ^ " "
      ^ (match op with Eq -> "==" | Ne -> "!=" | Lt -> "<")
      ^ " " ^ aexp_to_text b
  | And (a, b) -> "(" ^ bexp_to_text a ^ " && " ^ bexp_to_text b ^ ")"
  | Or (a, b) -> "(" ^ bexp_to_text a ^ " || " ^ bexp_to_text b ^ ")"
  | Done n -> "done $" ^ n

let fmt_offset n = if n >= 0 then "+" ^ string_of_int n else string_of_int n

let stmt_to_text = function
  | Assign (n, a) -> n ^ " <- " ^ aexp_to_text a
  | IfGoto (b, n) -> "if " ^ bexp_to_text b ^ " goto " ^ fmt_offset n
  | Goto n -> "goto " ^ fmt_offset n
  | Call n -> "call " ^ n
  | Run (n, id) -> "run " ^ n ^ " -> $" ^ id
  | Yield -> "yield"

let print code =
  String.concat "\n"
    (List.map
       (fun f ->
         "fn " ^ f.name ^ " {\n"
         ^ (if f.body = [] then ""
            else "  " ^ String.concat "; " (List.map stmt_to_text f.body) ^ "\n")
         ^ "}\n")
       code)

let parse_aexp =
  L.expression
    ~infix:
      [
        ("+", 1, 2, fun a b -> Bin (a, Add, b));
        ("-", 1, 2, fun a b -> Bin (a, Sub, b));
        ("*", 3, 4, fun a b -> Bin (a, Mul, b));
      ]
    ~atom:(fun read p ->
      match L.peek p with
      | L.Ident _ -> Atom (Var (L.identifier p))
      | L.Number _ -> Atom (Imm (L.natural p))
      | L.Symbol '(' -> L.group p ~opening:'(' read
      | _ -> L.error p "Unexpected token in arithmetic expression")

let parse_bexp =
  let relation p =
    L.binary p
      ~operators:[ ("==", Eq); ("!=", Ne); ("<", Lt) ]
      ~operand:parse_aexp
      ~make:(fun a op b -> Rel (a, op, b))
  in
  L.expression
    ~prefix:[ ("!", 5, fun b -> Not b) ]
    ~infix:
      [
        ("&&", 2, 3, fun a b -> And (a, b)); ("||", 0, 1, fun a b -> Or (a, b));
      ]
    ~atom:(fun read p ->
      if L.bump_if_keyword p "done" then (
        L.expect_symbol p '$';
        Done (L.identifier p))
      else if L.peek p = L.Symbol '(' then
        (* A group can contain a boolean expression or the arithmetic LHS of
           a relation. A failed group read leaves the parent cursor intact. *)
        try L.group p ~opening:'(' read with L.Error _ -> relation p
      else relation p)

let parse_offset p =
  let sign =
    if L.bump_if_symbol p '-' then -1
    else (
      ignore (L.bump_if_symbol p '+');
      1)
  in
  sign * L.integer p

let parse_stmt p =
  if L.bump_if_keyword p "if" then (
    let b = parse_bexp p in
    L.expect_keyword p "goto";
    IfGoto (b, parse_offset p))
  else if L.bump_if_keyword p "goto" then Goto (parse_offset p)
  else if L.bump_if_keyword p "call" then Call (L.identifier p)
  else if L.bump_if_keyword p "run" then (
    let name = L.identifier p in
    L.expect p "->";
    L.expect_symbol p '$';
    let id = L.identifier p in
    Run (name, id))
  else if L.bump_if_keyword p "yield" then Yield
  else
    let n = L.identifier p in
    L.expect p "<-";
    let a = parse_aexp p in
    Assign (n, a)

let parse =
  L.parse (fun p ->
      let code =
        L.many p (fun p ->
            L.expect_keyword p "fn";
            let name = L.identifier p in
            let body =
              L.group p ~opening:'{' (fun p ->
                  L.many ~skip_separator:';' p (fun p ->
                      let s = parse_stmt p in
                      if not (L.eof p) then L.expect_symbol p ';';
                      s))
            in
            { name; body })
      in
      if code = [] then L.error p "At least one function is required";
      code)
