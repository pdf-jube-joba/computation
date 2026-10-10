open Common

type aexp =
  | Var of string
  | Num of N.t
  | BinOp of aexp * abinop * aexp
  | IfThenElse of bexp * aexp * aexp

and bexp = Rel of aexp * relop * aexp | Or of bexp * bexp | Not of bexp
[@@deriving yojson]

type stmt =
  | Nop
  | Assign of string * aexp
  | Seq of stmt * stmt
  | If of bexp * stmt
  | While of bexp * stmt
[@@deriving yojson]

type code = stmt [@@deriving yojson]
type ainput = env [@@deriving yojson]
type foutput = env [@@deriving yojson]
type rinput = unit
type routput = unit
type state = { code : code; stmt : stmt; env : env } [@@deriving yojson]

let rec eval_aexp env = function
  | Var v -> env_get env v
  | Num n -> n
  | BinOp (a, op, b) ->
      (if op = Add then N.add else N.sub) (eval_aexp env a) (eval_aexp env b)
  | IfThenElse (c, a, b) -> eval_aexp env (if eval_bexp env c then a else b)

and eval_bexp env = function
  | Rel (a, op, b) -> (
      let c = N.compare (eval_aexp env a) (eval_aexp env b) in
      match op with Lt -> c < 0 | Eq -> c = 0 | Gt -> c > 0)
  | Or (a, b) -> eval_bexp env a || eval_bexp env b
  | Not b -> not (eval_bexp env b)

let rec small_step stmt env =
  match stmt with
  | Nop -> (Nop, env)
  | Assign (v, a) -> (Nop, set v (eval_aexp env a) env)
  | Seq (Nop, b) -> (b, env)
  | Seq (a, b) ->
      let a, env = small_step a env in
      (Seq (a, b), env)
  | If (c, b) -> ((if eval_bexp env c then b else Nop), env)
  | While (c, b) -> (If (c, Seq (b, While (c, b))), env)

let rec parse_aexp p =
  if L.eat_keyword p "if" then (
    let c = parse_bexp p in
    L.expect_keyword p "then";
    let a = parse_aexp p in
    L.expect_keyword p "else";
    let b = parse_aexp p in
    IfThenElse (c, a, b))
  else
    let a = parse_aatom p in
    let rec loop a =
      if L.eat_symbol p '+' then
        let b = parse_aatom p in
        loop (BinOp (a, Add, b))
      else if L.eat_symbol p '-' then
        let b = parse_aatom p in
        loop (BinOp (a, Sub, b))
      else a
    in
    loop a

and parse_aatom p =
  if L.eat_symbol p '(' then (
    let a = parse_aexp p in
    L.expect_symbol p ')';
    a)
  else match L.peek p with L.Number _ -> Num (number p) | _ -> Var (ident p)

and parse_bexp p =
  let a = parse_bnot p in
  let rec loop a =
    if L.eat p "||" then
      let b = parse_bnot p in
      loop (Or (a, b))
    else a
  in
  loop a

and parse_bnot p =
  if L.eat_symbol p '!' then Not (parse_bnot p)
  else if L.eat_symbol p '(' then (
    let b = parse_bexp p in
    L.expect_symbol p ')';
    b)
  else
    let a = parse_aexp p in
    let op =
      match L.next p with
      | L.Symbol '<' -> Lt
      | L.Symbol '=' -> Eq
      | L.Symbol '>' -> Gt
      | _ -> L.error p "Expected relation operator"
    in
    let b = parse_aexp p in
    Rel (a, op, b)

let rec parse_stmt p =
  let a = parse_single p in
  let rec loop a =
    if L.eat_symbol p ';' then
      let b = parse_single p in
      loop (Seq (a, b))
    else a
  in
  loop a

and parse_single p =
  if L.eat_keyword p "Nop" then Nop
  else if L.eat_keyword p "if" then
    let c = parse_bexp p in
    let b = parse_single p in
    If (c, b)
  else if L.eat_keyword p "while" then (
    let c = parse_bexp p in
    L.expect_symbol p '[';
    let b = parse_stmt p in
    L.expect_symbol p ']';
    While (c, b))
  else if L.eat_symbol p '(' then (
    let b = parse_stmt p in
    L.expect_symbol p ')';
    b)
  else
    let v = ident p in
    L.expect p ":=";
    Assign (v, parse_aexp p)

let parse_code = L.parse parse_stmt
let parse_ainput = parse_env
let parse_rinput = Utils.Machine.empty_input

let rec aexp_to_text = function
  | Var v -> v
  | Num n -> N.to_string n
  | BinOp (a, op, b) ->
      "(" ^ aexp_to_text a
      ^ (if op = Add then " + " else " - ")
      ^ aexp_to_text b ^ ")"
  | IfThenElse (c, a, b) ->
      "if " ^ bexp_to_text c ^ " then " ^ aexp_to_text a ^ " else "
      ^ aexp_to_text b

and bexp_to_text = function
  | Rel (a, op, b) ->
      "(" ^ aexp_to_text a ^ " " ^ rel_text op ^ " " ^ aexp_to_text b ^ ")"
  | Or (a, b) -> "(" ^ bexp_to_text a ^ " || " ^ bexp_to_text b ^ ")"
  | Not b -> "!" ^ bexp_to_text b

let rec stmt_to_text = function
  | Nop -> "Nop"
  | Assign (v, a) -> v ^ " := " ^ aexp_to_text a
  | Seq (a, b) -> stmt_to_text a ^ " ; " ^ stmt_to_text b
  | If (c, b) -> "if " ^ bexp_to_text c ^ " " ^ stmt_to_text b
  | While (c, b) -> "while " ^ bexp_to_text c ^ " [ " ^ stmt_to_text b ^ " ]"

let print = stmt_to_text
let make code env = Ok { code; stmt = code; env }

let step s () =
  if s.stmt = Nop then Ok (Utils.Machine.Halt s.env)
  else
    let stmt, env = small_step s.stmt s.env in
    Ok (Utils.Machine.Continue { next = { s with stmt; env }; output = () })

let snapshot = state_to_yojson
let restore = state_of_yojson

let render s =
  Utils.Render.state
    [
      Utils.Render.text ~title:"stmt" (stmt_to_text s.stmt);
      Utils.Render.table ~title:"env"
        ~columns:[ Utils.Render.text "var"; Utils.Render.text "value" ]
        (List.map
           (fun (k, v) ->
             [ Utils.Render.text k; Utils.Render.text (N.to_string v) ])
           s.env);
    ]

let routput_json = unit_json
let foutput_json e = `String (print_env e)

module Machine = struct
  type nonrec code = code
  type nonrec ainput = ainput
  type nonrec rinput = rinput
  type nonrec routput = routput
  type nonrec foutput = foutput
  type nonrec state = state

  let parse_code = parse_code
  let parse_ainput = parse_ainput
  let parse_rinput = parse_rinput
  let make = make
  let step = step
  let snapshot = snapshot
  let restore = restore
  let render = render
  let routput_json = routput_json
  let foutput_json = foutput_json
end
