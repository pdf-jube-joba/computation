module L = Utils.Lexer

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

module Code = struct
  type t = code

  let reserved =
    [ "Nop"; "if"; "then"; "end"; "while"; "is_empty"; "head"; "tail" ]

  let parse_name p =
    let n = L.identifier p in
    if List.mem n reserved then failwith "Expected identifier";
    n

  let rec parse_aexp p =
    let a = parse_prefix p in
    let rec loop a =
      if L.eat p "++" then
        let b = parse_prefix p in
        loop (Concat (a, b))
      else a
    in
    loop a

  and parse_prefix p =
    if L.eat_keyword p "head" then Head (parse_prefix p)
    else if L.eat_keyword p "tail" then Tail (parse_prefix p)
    else
      match L.next p with
      | L.Ident n when not (List.mem n reserved) -> Var n
      | L.Symbol '(' ->
          let a = parse_aexp p in
          L.expect_symbol p ')';
          a
      | L.Symbol '[' ->
          Const
            (L.separated_until p ~separator:',' ~closing:']' L.character_literal)
      | _ -> failwith "Invalid aexp"

  let parse_bexp p =
    if L.eat_keyword p "is_empty" then IsEmpty (parse_aexp p)
    else
      let a = parse_aexp p in
      L.expect p "==";
      let b = parse_aexp p in
      Eq (a, b)

  let rec parse_stmt p =
    let s = parse_single p in
    let rec loop s =
      if L.eat_symbol p ';' then
        let b = parse_single p in
        loop (Seq (s, b))
      else s
    in
    loop s

  and parse_single p =
    if L.eat_keyword p "Nop" then Nop
    else if L.eat_keyword p "if" then (
      let c = parse_bexp p in
      L.expect_keyword p "then";
      let b = parse_stmt p in
      L.expect_keyword p "end";
      If (c, b))
    else if L.eat_keyword p "while" then (
      let c = parse_bexp p in
      L.expect_symbol p '{';
      let b = parse_stmt p in
      L.expect_symbol p '}';
      While (c, b))
    else if L.eat_symbol p '(' then (
      let b = parse_stmt p in
      L.expect_symbol p ')';
      b)
    else
      let n = parse_name p in
      L.expect p ":=";
      let a = parse_aexp p in
      Assign (n, a)

  let parse = L.parse ~character_literals:true parse_stmt

  let rec aexp_to_text = function
    | Const cs ->
        "[" ^ String.concat ", " (List.map Utils.Char_literal.to_text cs) ^ "]"
    | Var n -> n
    | Concat (a, b) -> "(" ^ aexp_to_text a ^ " ++ " ^ aexp_to_text b ^ ")"
    | Head a -> "head " ^ aexp_to_text a
    | Tail a -> "tail " ^ aexp_to_text a

  let bexp_to_text = function
    | IsEmpty a -> "is_empty " ^ aexp_to_text a
    | Eq (a, b) -> aexp_to_text a ^ " == " ^ aexp_to_text b

  let rec print = function
    | Nop -> "Nop"
    | Assign (n, a) -> n ^ " := " ^ aexp_to_text a
    | Seq (a, b) -> print a ^ " ; " ^ print b
    | If (c, b) -> "if " ^ bexp_to_text c ^ " then " ^ print b ^ " end"
    | While (c, b) -> "while " ^ bexp_to_text c ^ " { " ^ print b ^ " }"
end

module Ainput = struct
  type t = ainput

  let print_value cs = Code.aexp_to_text (Const cs)

  let parse_value =
    L.parse ~character_literals:true (fun p ->
        match Code.parse_aexp p with
        | Const cs -> cs
        | _ -> failwith "Expected constant character list")

  let parse = Utils.Environment.parse parse_value
  let print = Utils.Environment.print ~trailing_newline:true print_value
end

let rec eval_aexp env = function
  | Const cs -> cs
  | Var n -> Option.value ~default:[] (List.assoc_opt n env)
  | Concat (a, b) -> eval_aexp env a @ eval_aexp env b
  | Head a -> ( match eval_aexp env a with [] -> [] | c :: _ -> [ c ])
  | Tail a -> ( match eval_aexp env a with [] -> [] | _ :: cs -> cs)

let eval_bexp env = function
  | IsEmpty a -> eval_aexp env a = []
  | Eq (a, b) -> eval_aexp env a = eval_aexp env b

let rec small_step stmt env =
  match stmt with
  | Nop -> (Nop, env)
  | Assign (n, a) -> (Nop, Utils.Assoc.set n (eval_aexp env a) env)
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
  Utils.Render.statement_env (Code.print s.stmt) Ainput.print_value s.env

module Rinput = Utils.Text.Unit
module Routput = Utils.Text.Unit
module Foutput = Ainput
