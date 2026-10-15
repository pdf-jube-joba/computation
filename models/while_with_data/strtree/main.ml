module L = Utils_parsing.Parser

type aexp = Mtoa of texp | AtomChar of string

and texp =
  | Var of Utils_names.Identifier.t
  | Atom of aexp
  | Cons of texp * texp
  | Left of texp
  | Right of texp
[@@deriving yojson]

type bexp = IsAtom of texp | Eq of texp * texp [@@deriving yojson]

type stmt =
  | Nop
  | Assign of Utils_names.Identifier.t * texp
  | Seq of stmt * stmt
  | IfEq of bexp * stmt
  | While of bexp * stmt
[@@deriving yojson]

type value = Utils_data.Character_tree.t =
  | AtomValue of string
  | ConsValue of value * value
[@@deriving yojson]

type code = stmt [@@deriving yojson]
type environment = (Utils_names.Identifier.t * value) list [@@deriving yojson]
type ainput = environment
type rinput = unit
type routput = unit
type foutput = environment
type state = { code : code; stmt : stmt; env : environment } [@@deriving yojson]

module Code = struct
  type t = code

  let rec aexp_to_text = function
    | Mtoa t -> "mtoa " ^ texp_to_text t
    | AtomChar c -> Utils_parsing.Char_literal.to_text c

  and texp_to_text = function
    | Var n -> Utils_names.Identifier.to_string n
    | Atom a -> "atom " ^ aexp_to_text a
    | Cons (a, b) -> "cons " ^ texp_to_text a ^ " " ^ texp_to_text b
    | Left a -> "left " ^ texp_to_text a
    | Right a -> "right " ^ texp_to_text a

  let bexp_to_text = function
    | IsAtom a -> "is_atom " ^ texp_to_text a
    | Eq (a, b) -> "(" ^ texp_to_text a ^ " == " ^ texp_to_text b ^ ")"

  let rec print = function
    | Nop -> "Nop"
    | Assign (n, a) ->
        Utils_names.Identifier.to_string n ^ " := " ^ texp_to_text a
    | Seq (a, b) -> print a ^ " ; " ^ print b
    | IfEq (c, b) -> "ifeq " ^ bexp_to_text c ^ " then " ^ print b ^ " end"
    | While (c, b) -> "while " ^ bexp_to_text c ^ " { " ^ print b ^ " }"

  let reserved =
    [
      "Nop";
      "ifeq";
      "then";
      "end";
      "while";
      "is_atom";
      "mtoa";
      "atom";
      "cons";
      "left";
      "right";
    ]

  let parse_name p =
    let n = L.identifier p in
    if List.mem (Utils_names.Identifier.to_string n) reserved then
      failwith "Expected identifier";
    n

  let rec parse_aexp p =
    if L.bump_if_keyword p "mtoa" then Mtoa (parse_texp p)
    else if L.is_group p L.Parentheses then L.group p L.Parentheses parse_aexp
    else AtomChar (L.character_literal p)

  and parse_texp p =
    if L.bump_if_keyword p "atom" then Atom (parse_aexp p)
    else if L.bump_if_keyword p "cons" then
      let a = parse_texp p in
      let b = parse_texp p in
      Cons (a, b)
    else if L.bump_if_keyword p "left" then Left (parse_texp p)
    else if L.bump_if_keyword p "right" then Right (parse_texp p)
    else if L.is_group p L.Parentheses then L.group p L.Parentheses parse_texp
    else Var (parse_name p)

  let parse_bexp p =
    if L.bump_if_keyword p "is_atom" then IsAtom (parse_texp p)
    else
      (* Rust's printer parenthesizes equality. Accept that printed form too. *)
      let relation p =
        L.binary p
          ~operators:[ ("==", ()) ]
          ~operand:parse_texp
          ~make:(fun a () b -> Eq (a, b))
      in
      match L.attempt p relation with
      | Ok c -> c
      | Error _ -> L.group p L.Parentheses relation

  let rec parse_stmt p =
    L.chain_left p ~separator:';' ~combine:(fun a b -> Seq (a, b)) parse_single

  and parse_single p =
    if L.bump_if_keyword p "Nop" then Nop
    else if L.bump_if_keyword p "ifeq" then (
      let c = parse_bexp p in
      L.expect_keyword p "then";
      let b = parse_stmt p in
      L.expect_keyword p "end";
      IfEq (c, b))
    else if L.bump_if_keyword p "while" then
      let c = parse_bexp p in
      let b = L.group p L.Braces parse_stmt in
      While (c, b)
    else if L.is_group p L.Parentheses then L.group p L.Parentheses parse_stmt
    else
      let n = parse_name p in
      L.expect p ":=";
      let a = parse_texp p in
      Assign (n, a)

  let parse = L.parse ~character_literals:true parse_stmt
end

module Ainput = struct
  type t = ainput

  let print_value = Utils_codec.Character_tree_text.print
  let parse_value = Utils_codec.Character_tree_text.parse
  let parse = Utils_codec.Environment_text.parse parse_value

  let print =
    Utils_codec.Environment_text.print ~trailing_newline:true print_value
end

let rec eval_aexp env = function
  | AtomChar c -> c
  | Mtoa t -> (
      match eval_texp env t with
      | AtomValue c -> c
      | _ -> failwith "mtoa expects an atom")

and eval_texp env = function
  | Var n ->
      Utils_base.Assoc.lookup n env
        ("uninitialized variable: " ^ Utils_names.Identifier.to_string n)
  | Atom a -> AtomValue (eval_aexp env a)
  | Cons (a, b) ->
      let a = eval_texp env a in
      let b = eval_texp env b in
      ConsValue (a, b)
  | Left t -> (
      match eval_texp env t with
      | ConsValue (a, _) -> a
      | _ -> failwith "left expects a cons cell")
  | Right t -> (
      match eval_texp env t with
      | ConsValue (_, b) -> b
      | _ -> failwith "right expects a cons cell")

let eval_bexp env = function
  | IsAtom t -> ( match eval_texp env t with AtomValue _ -> true | _ -> false)
  | Eq (a, b) ->
      let a = eval_texp env a in
      let b = eval_texp env b in
      a = b

let rec small_step stmt env =
  match stmt with
  | Nop -> (Nop, env)
  | Assign (n, a) -> (Nop, Utils_base.Assoc.set n (eval_texp env a) env)
  | Seq (Nop, b) -> (b, env)
  | Seq (a, b) ->
      let a, env = small_step a env in
      (Seq (a, b), env)
  | IfEq (c, b) -> ((if eval_bexp env c then b else Nop), env)
  | While (c, b) -> (IfEq (c, Seq (b, While (c, b))), env)

let make code env = Ok { code; stmt = code; env }

let step s () =
  Utils_base.Error.protect (fun () ->
      if s.stmt = Nop then Utils_runtime.Machine.Halt s.env
      else
        let stmt, env = small_step s.stmt s.env in
        Utils_runtime.Machine.Continue
          { next = { s with stmt; env }; output = () })

let snapshot = state_to_yojson
let restore = state_of_yojson

let render s =
  Utils_runtime.Render.statement_env ~name_text:Utils_names.Identifier.to_string
    (Code.print s.stmt) Ainput.print_value s.env

module Rinput = Utils_codec.Text.Unit
module Routput = Utils_codec.Text.Unit
module Foutput = Ainput
