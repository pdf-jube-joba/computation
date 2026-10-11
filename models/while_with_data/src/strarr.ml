open Data_parser

type aexp =
  | Const of string list
  | Var of Utils.Identifier.t
  | Concat of aexp * aexp
  | Head of aexp
  | Tail of aexp
[@@deriving yojson]

type bexp = IsEmpty of aexp | Eq of aexp * aexp [@@deriving yojson]

type stmt =
  | Nop
  | Assign of Utils.Identifier.t * aexp
  | Seq of stmt * stmt
  | If of bexp * stmt
  | While of bexp * stmt
[@@deriving yojson]

type code = stmt [@@deriving yojson]
type environment = (Utils.Identifier.t * string list) list [@@deriving yojson]
type ainput = environment
type rinput = unit
type routput = unit
type foutput = environment
type state = { code : code; stmt : stmt; env : environment } [@@deriving yojson]

let reserved =
  [ "Nop"; "if"; "then"; "end"; "while"; "is_empty"; "head"; "tail" ]

let parse_name p =
  let n = ident p in
  if List.mem n reserved then fail "Expected identifier";
  n

let rec parse_aexp p =
  let a = parse_prefix p in
  let rec loop a =
    if eat p (Op "++") then
      let b = parse_prefix p in
      loop (Concat (a, b))
    else a
  in
  loop a

and parse_prefix p =
  if eat p (Word "head") then Head (parse_prefix p)
  else if eat p (Word "tail") then Tail (parse_prefix p)
  else
    match next p with
    | Word n when not (List.mem n reserved) -> Var n
    | Sym '(' ->
        let a = parse_aexp p in
        expect p (Sym ')');
        a
    | Sym '[' ->
        let rec chars acc =
          let c = character p in
          if eat p (Sym ']') then List.rev (c :: acc)
          else (
            expect p (Sym ',');
            chars (c :: acc))
        in
        Const (if eat p (Sym ']') then [] else chars [])
    | _ -> fail "Invalid aexp"

let parse_bexp p =
  if eat p (Word "is_empty") then IsEmpty (parse_aexp p)
  else
    let a = parse_aexp p in
    expect p (Op "==");
    let b = parse_aexp p in
    Eq (a, b)

let rec parse_stmt p =
  let s = parse_single p in
  let rec loop s =
    if eat p (Sym ';') then
      let b = parse_single p in
      loop (Seq (s, b))
    else s
  in
  loop s

and parse_single p =
  if eat p (Word "Nop") then Nop
  else if eat p (Word "if") then (
    let c = parse_bexp p in
    expect p (Word "then");
    let b = parse_stmt p in
    expect p (Word "end");
    If (c, b))
  else if eat p (Word "while") then (
    let c = parse_bexp p in
    expect p (Sym '{');
    let b = parse_stmt p in
    expect p (Sym '}');
    While (c, b))
  else if eat p (Sym '(') then (
    let b = parse_stmt p in
    expect p (Sym ')');
    b)
  else
    let n = parse_name p in
    expect p (Op ":=");
    let a = parse_aexp p in
    Assign (n, a)

let parse_code = parse parse_stmt

module Ainput = struct
  type t = ainput

  let print_value cs =
    "[" ^ String.concat ", " (List.map Utils.Char_literal.to_text cs) ^ "]"

  let parse_value =
    parse (fun p ->
        match parse_aexp p with
        | Const cs -> cs
        | _ -> fail "Expected constant character list")

  let parse = Utils.Environment.parse parse_value
  let print = Utils.Environment.print ~trailing_newline:true print_value
end

let rec aexp_to_text = function
  | Const cs -> Ainput.print_value cs
  | Var n -> n
  | Concat (a, b) -> "(" ^ aexp_to_text a ^ " ++ " ^ aexp_to_text b ^ ")"
  | Head a -> "head " ^ aexp_to_text a
  | Tail a -> "tail " ^ aexp_to_text a

let bexp_to_text = function
  | IsEmpty a -> "is_empty " ^ aexp_to_text a
  | Eq (a, b) -> aexp_to_text a ^ " == " ^ aexp_to_text b

let rec stmt_to_text = function
  | Nop -> "Nop"
  | Assign (n, a) -> n ^ " := " ^ aexp_to_text a
  | Seq (a, b) -> stmt_to_text a ^ " ; " ^ stmt_to_text b
  | If (c, b) -> "if " ^ bexp_to_text c ^ " then " ^ stmt_to_text b ^ " end"
  | While (c, b) -> "while " ^ bexp_to_text c ^ " { " ^ stmt_to_text b ^ " }"

let print = stmt_to_text
let get env n = Option.value ~default:[] (List.assoc_opt n env)

let rec eval_aexp env = function
  | Const cs -> cs
  | Var n -> get env n
  | Concat (a, b) -> eval_aexp env a @ eval_aexp env b
  | Head a -> ( match eval_aexp env a with [] -> [] | c :: _ -> [ c ])
  | Tail a -> ( match eval_aexp env a with [] -> [] | _ :: cs -> cs)

let eval_bexp env = function
  | IsEmpty a -> eval_aexp env a = []
  | Eq (a, b) -> eval_aexp env a = eval_aexp env b

let rec small_step stmt env =
  match stmt with
  | Nop -> (Nop, env)
  | Assign (n, a) -> (Nop, set env n (eval_aexp env a))
  | Seq (Nop, b) -> (b, env)
  | Seq (a, b) ->
      let a, env = small_step a env in
      (Seq (a, b), env)
  | If (c, b) -> ((if eval_bexp env c then b else Nop), env)
  | While (c, b) -> (If (c, Seq (b, While (c, b))), env)

let make code env = Ok { code; stmt = code; env }

let step s () =
  if s.stmt = Nop then Ok (Utils.Machine.Halt s.env)
  else
    let stmt, env = small_step s.stmt s.env in
    Ok (Utils.Machine.Continue { next = { s with stmt; env }; output = () })

let snapshot = state_to_yojson
let restore = state_of_yojson

let render s =
  Utils.Render.statement_env (stmt_to_text s.stmt) Ainput.print_value s.env

module Rinput = Utils.Text.Unit
module Routput = Utils.Text.Unit
module Foutput = Ainput

module Machine = struct
  module Ainput = Ainput
  module Rinput = Rinput
  module Routput = Routput
  module Foutput = Foutput

  type nonrec code = code
  type nonrec state = state

  let parse_code = parse_code
  let make = make
  let step = step
  let snapshot = snapshot
  let restore = restore
  let render = render
end
