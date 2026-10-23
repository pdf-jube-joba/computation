open Structured_lang_shared.Arithmetic

type aexp =
  | Var of Utils_names.Identifier.t
  | Num of N.t
  | BinOp of aexp * abinop * aexp
  | IfThenElse of bexp * aexp * aexp

and bexp = Rel of aexp * relop * aexp | Or of bexp * bexp | Not of bexp
[@@deriving yojson]

type stmt =
  | Nop
  | Assign of Utils_names.Identifier.t * aexp
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
  | BinOp (a, op, b) -> eval_abinop op (eval_aexp env a) (eval_aexp env b)
  | IfThenElse (c, a, b) -> eval_aexp env (if eval_bexp env c then a else b)

and eval_bexp env = function
  | Rel (a, op, b) -> eval_relop op (eval_aexp env a) (eval_aexp env b)
  | Or (a, b) -> eval_bexp env a || eval_bexp env b
  | Not b -> not (eval_bexp env b)

let rec small_step stmt env =
  match stmt with
  | Nop -> (Nop, env)
  | Assign (v, a) -> (Nop, Utils_base.Assoc.set v (eval_aexp env a) env)
  | Seq (Nop, b) -> (b, env)
  | Seq (a, b) ->
      let a, env = small_step a env in
      (Seq (a, b), env)
  | If (c, b) -> ((if eval_bexp env c then b else Nop), env)
  | While (c, b) -> (If (c, Seq (b, While (c, b))), env)

module Code = struct
  type t = code

  let rec parse_aexp p =
    if L.bump_if_keyword p "if" then (
      let c = parse_bexp p in
      L.expect_keyword p "then";
      let a = parse_aexp p in
      L.expect_keyword p "else";
      let b = parse_aexp p in
      IfThenElse (c, a, b))
    else
      L.expression
        ~infix:
          [
            ("+", 1, 2, fun a b -> BinOp (a, Add, b));
            ("-", 1, 2, fun a b -> BinOp (a, Sub, b));
          ]
        ~atom:(fun _ p -> parse_aatom p)
        p

  and parse_aatom p =
    if L.is_group p L.Parentheses then L.group p L.Parentheses parse_aexp
    else
      match L.peek p with
      | L.Number _ -> Num (L.natural p)
      | _ -> Var (L.identifier p)

  and parse_bexp p =
    let relation p =
      L.binary p
        ~operators:[ ("<", Lt); ("=", Eq); (">", Gt) ]
        ~operand:parse_aexp
        ~make:(fun a op b -> Rel (a, op, b))
    in
    L.expression
      ~prefix:[ ("!", 2, fun b -> Not b) ]
      ~infix:[ ("||", 0, 1, fun a b -> Or (a, b)) ]
      ~atom:(fun read p ->
        if L.is_group p L.Parentheses then
          try L.group p L.Parentheses read with L.Error _ -> relation p
        else relation p)
      p

  let rec parse_stmt p =
    L.chain_left p ~separator:';' ~combine:(fun a b -> Seq (a, b)) parse_single

  and parse_single p =
    if L.bump_if_keyword p "Nop" then Nop
    else if L.bump_if_keyword p "if" then
      let c = parse_bexp p in
      let b = parse_single p in
      If (c, b)
    else if L.bump_if_keyword p "while" then
      let c = parse_bexp p in
      let b = L.group p L.Brackets parse_stmt in
      While (c, b)
    else if L.is_group p L.Parentheses then L.group p L.Parentheses parse_stmt
    else
      let v = L.identifier p in
      L.expect p ":=";
      Assign (v, parse_aexp p)

  let parse = L.parse parse_stmt

  let rec aexp_to_text = function
    | Var v -> Utils_names.Identifier.to_string v
    | Num n -> N.to_string n
    | BinOp (a, op, b) ->
        "(" ^ aexp_to_text a ^ " " ^ abinop_text op ^ " " ^ aexp_to_text b ^ ")"
    | IfThenElse (c, a, b) ->
        "\\if " ^ bexp_to_text c ^ " \\then " ^ aexp_to_text a ^ " \\else "
        ^ aexp_to_text b

  and bexp_to_text = function
    | Rel (a, op, b) ->
        "(" ^ aexp_to_text a ^ " " ^ rel_text op ^ " " ^ aexp_to_text b ^ ")"
    | Or (a, b) -> "(" ^ bexp_to_text a ^ " || " ^ bexp_to_text b ^ ")"
    | Not b -> "!" ^ bexp_to_text b

  let rec print = function
    | Nop -> "\\Nop"
    | Assign (v, a) ->
        Utils_names.Identifier.to_string v ^ " := " ^ aexp_to_text a
    | Seq (a, b) -> print a ^ " ; " ^ print b
    | If (c, b) -> "\\if " ^ bexp_to_text c ^ " " ^ print b
    | While (c, b) -> "\\while " ^ bexp_to_text c ^ " [ " ^ print b ^ " ]"
end

let make code env = Ok { code; stmt = code; env }

let step s () =
  if s.stmt = Nop then Ok (Utils_runtime.Machine.Halt s.env)
  else
    let stmt, env = small_step s.stmt s.env in
    Ok
      (Utils_runtime.Machine.Continue
         { next = { s with stmt; env }; output = () })

module Snapshot = struct
  type t = state

  let to_yojson = state_to_yojson
  let of_yojson = state_of_yojson
end

let render s =
  Utils_runtime.Render.statement_env ~name_text:Utils_names.Identifier.to_string
    (Code.print s.stmt) N.to_string s.env

module Ainput = Structured_lang_shared.Arithmetic.Env
module Rinput = Utils_data.Text.Unit
module Routput = Utils_data.Text.Unit
module Foutput = Ainput

let rec a_vars = function
  | Var v -> [ v ]
  | Num _ -> []
  | BinOp (a, _, b) -> a_vars a @ a_vars b
  | IfThenElse (c, a, b) -> b_vars c @ a_vars a @ a_vars b

and b_vars = function
  | Rel (a, _, b) -> a_vars a @ a_vars b
  | Or (a, b) -> b_vars a @ b_vars b
  | Not b -> b_vars b

let rec stmt_vars = function
  | Nop -> []
  | Assign (v, a) -> v :: a_vars a
  | Seq (a, b) -> stmt_vars a @ stmt_vars b
  | If (c, b) | While (c, b) -> b_vars c @ stmt_vars b
