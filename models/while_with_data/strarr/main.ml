module L = Utils_parsing.Parser

type aexp =
  | Const of string list
  | Var of Utils_names.Identifier.t
  | Concat of aexp * aexp
  | Head of aexp
  | Tail of aexp
[@@deriving yojson]

type bexp = IsEmpty of aexp | Eq of aexp * aexp [@@deriving yojson]

type stmt =
  | Nop
  | Assign of Utils_names.Identifier.t * aexp
  | Seq of stmt * stmt
  | If of bexp * stmt
  | While of bexp * stmt
[@@deriving yojson]

type code = stmt [@@deriving yojson]

type environment = (Utils_names.Identifier.t * string list) list
[@@deriving yojson]

type ainput = environment
type rinput = unit
type routput = unit
type foutput = environment
type state = { code : code; stmt : stmt; env : environment } [@@deriving yojson]

module Code = struct
  type t = code

  let parse_name = L.identifier

  let parse_aexp =
    L.expression
      ~prefix:[ ("\\head", 2, fun a -> Head a); ("\\tail", 2, fun a -> Tail a) ]
      ~infix:[ ("++", 1, 2, fun a b -> Concat (a, b)) ]
      ~atom:(fun read p ->
        match L.peek p with
        | L.Ident _ -> Var (L.identifier p)
        | _ when L.is_group p L.Parentheses -> L.group p L.Parentheses read
        | _ when L.is_group p L.Brackets ->
            Const
              (L.group p L.Brackets (fun p ->
                   L.separated_list p ~separator:',' L.character_literal))
        | _ -> L.error p "Invalid aexp")

  let parse_bexp p =
    if L.bump_if_keyword p "is_empty" then IsEmpty (parse_aexp p)
    else
      L.binary p
        ~operators:[ ("==", ()) ]
        ~operand:parse_aexp
        ~make:(fun a () b -> Eq (a, b))

  let rec parse_stmt p =
    L.chain_left p ~separator:';' ~combine:(fun a b -> Seq (a, b)) parse_single

  and parse_single p =
    if L.bump_if_keyword p "Nop" then Nop
    else if L.bump_if_keyword p "if" then (
      let c = parse_bexp p in
      L.expect_keyword p "then";
      let b = parse_stmt p in
      L.expect_keyword p "end";
      If (c, b))
    else if L.bump_if_keyword p "while" then
      let c = parse_bexp p in
      let b = L.group p L.Braces parse_stmt in
      While (c, b)
    else if L.is_group p L.Parentheses then L.group p L.Parentheses parse_stmt
    else
      let n = parse_name p in
      L.expect p ":=";
      let a = parse_aexp p in
      Assign (n, a)

  let parse = L.parse ~character_literals:true parse_stmt

  let rec aexp_to_text = function
    | Const cs ->
        "["
        ^ String.concat ", " (List.map Utils_parsing.Char_literal.to_text cs)
        ^ "]"
    | Var n -> Utils_names.Identifier.to_string n
    | Concat (a, b) -> "(" ^ aexp_to_text a ^ " ++ " ^ aexp_to_text b ^ ")"
    | Head a -> "\\head " ^ aexp_to_text a
    | Tail a -> "\\tail " ^ aexp_to_text a

  let bexp_to_text = function
    | IsEmpty a -> "\\is_empty " ^ aexp_to_text a
    | Eq (a, b) -> aexp_to_text a ^ " == " ^ aexp_to_text b

  let rec print = function
    | Nop -> "\\Nop"
    | Assign (n, a) ->
        Utils_names.Identifier.to_string n ^ " := " ^ aexp_to_text a
    | Seq (a, b) -> print a ^ " ; " ^ print b
    | If (c, b) -> "\\if " ^ bexp_to_text c ^ " \\then " ^ print b ^ " \\end"
    | While (c, b) -> "\\while " ^ bexp_to_text c ^ " { " ^ print b ^ " }"
end

module Ainput = struct
  type t = ainput

  let print_value = Utils_data.Character_list.print
  let parse_value = Utils_data.Character_list.parse
  let parse = Utils_data.Environment.parse parse_value
  let print = Utils_data.Environment.print ~trailing_newline:true print_value
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
  | Assign (n, a) -> (Nop, Utils_base.Assoc.set n (eval_aexp env a) env)
  | Seq (Nop, b) -> (b, env)
  | Seq (a, b) ->
      let a, env = small_step a env in
      (Seq (a, b), env)
  | If (c, b) -> ((if eval_bexp env c then b else Nop), env)
  | While (c, b) -> (If (c, Seq (b, While (c, b))), env)

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
    (Code.print s.stmt) Ainput.print_value s.env

module Rinput = Utils_data.Text.Unit
module Routput = Utils_data.Text.Unit
module Foutput = Ainput
