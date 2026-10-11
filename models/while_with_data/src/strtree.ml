module L = Utils.Lexer

type aexp = Mtoa of texp | AtomChar of string

and texp =
  | Var of Utils.Identifier.t
  | Atom of aexp
  | Cons of texp * texp
  | Left of texp
  | Right of texp
[@@deriving yojson]

type bexp = IsAtom of texp | Eq of texp * texp [@@deriving yojson]

type stmt =
  | Nop
  | Assign of Utils.Identifier.t * texp
  | Seq of stmt * stmt
  | IfEq of bexp * stmt
  | While of bexp * stmt
[@@deriving yojson]

type value = AtomValue of string | ConsValue of value * value
[@@deriving yojson]

type code = stmt [@@deriving yojson]
type environment = (Utils.Identifier.t * value) list [@@deriving yojson]
type ainput = environment
type rinput = unit
type routput = unit
type foutput = environment
type state = { code : code; stmt : stmt; env : environment } [@@deriving yojson]

module Code = struct
  type t = code

  let rec aexp_to_text = function
    | Mtoa t -> "mtoa " ^ texp_to_text t
    | AtomChar c -> Utils.Char_literal.to_text c

  and texp_to_text = function
    | Var n -> n
    | Atom a -> "atom " ^ aexp_to_text a
    | Cons (a, b) -> "cons " ^ texp_to_text a ^ " " ^ texp_to_text b
    | Left a -> "left " ^ texp_to_text a
    | Right a -> "right " ^ texp_to_text a

  let bexp_to_text = function
    | IsAtom a -> "is_atom " ^ texp_to_text a
    | Eq (a, b) -> "(" ^ texp_to_text a ^ " == " ^ texp_to_text b ^ ")"

  let rec print = function
    | Nop -> "Nop"
    | Assign (n, a) -> n ^ " := " ^ texp_to_text a
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
    if List.mem n reserved then failwith "Expected identifier";
    n

  let rec parse_aexp p =
    if L.eat_keyword p "mtoa" then Mtoa (parse_texp p)
    else if L.eat_symbol p '(' then (
      let a = parse_aexp p in
      L.expect_symbol p ')';
      a)
    else AtomChar (L.character_literal p)

  and parse_texp p =
    if L.eat_keyword p "atom" then Atom (parse_aexp p)
    else if L.eat_keyword p "cons" then
      let a = parse_texp p in
      let b = parse_texp p in
      Cons (a, b)
    else if L.eat_keyword p "left" then Left (parse_texp p)
    else if L.eat_keyword p "right" then Right (parse_texp p)
    else if L.eat_symbol p '(' then (
      let a = parse_texp p in
      L.expect_symbol p ')';
      a)
    else Var (parse_name p)

  let parse_bexp p =
    if L.eat_keyword p "is_atom" then IsAtom (parse_texp p)
    else
      (* Rust's printer parenthesizes equality. Accept that printed form too. *)
      let saved = p.L.pos in
      let relation () =
        let a = parse_texp p in
        L.expect p "==";
        let b = parse_texp p in
        Eq (a, b)
      in
      try relation ()
      with Failure _ | L.Error _ ->
        p.L.pos <- saved;
        L.expect_symbol p '(';
        let c = relation () in
        L.expect_symbol p ')';
        c

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
    else if L.eat_keyword p "ifeq" then (
      let c = parse_bexp p in
      L.expect_keyword p "then";
      let b = parse_stmt p in
      L.expect_keyword p "end";
      IfEq (c, b))
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
      let a = parse_texp p in
      Assign (n, a)

  let parse = L.parse ~character_literals:true parse_stmt
end

module Ainput = struct
  type t = ainput

  let rec print_value = function
    | AtomValue c -> "atom " ^ Utils.Char_literal.to_text c
    | ConsValue (a, b) -> "cons " ^ print_value a ^ " " ^ print_value b

  let rec lower_value = function
    | Atom (AtomChar c) -> AtomValue c
    | Cons (a, b) -> ConsValue (lower_value a, lower_value b)
    | Var n -> failwith ("unexpected variable in value: " ^ n)
    | Atom (Mtoa _) -> failwith "unexpected mtoa in value"
    | Left _ | Right _ -> failwith "unexpected projection in value"

  let parse_value =
    L.parse ~character_literals:true (fun p -> lower_value (Code.parse_texp p))

  let parse = Utils.Environment.parse parse_value
  let print = Utils.Environment.print ~trailing_newline:true print_value
end

let rec eval_aexp env = function
  | AtomChar c -> c
  | Mtoa t -> (
      match eval_texp env t with
      | AtomValue c -> c
      | _ -> failwith "mtoa expects an atom")

and eval_texp env = function
  | Var n -> Utils.Assoc.lookup n env ("uninitialized variable: " ^ n)
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
  | Assign (n, a) -> (Nop, Utils.Assoc.set n (eval_texp env a) env)
  | Seq (Nop, b) -> (b, env)
  | Seq (a, b) ->
      let a, env = small_step a env in
      (Seq (a, b), env)
  | IfEq (c, b) -> ((if eval_bexp env c then b else Nop), env)
  | While (c, b) -> (IfEq (c, Seq (b, While (c, b))), env)

let make code env = Ok { code; stmt = code; env }

let step s () =
  Utils.Lexer.protect (fun () ->
      if s.stmt = Nop then Utils.Machine.Halt s.env
      else
        let stmt, env = small_step s.stmt s.env in
        Utils.Machine.Continue { next = { s with stmt; env }; output = () })

let snapshot = state_to_yojson
let restore = state_of_yojson

let render s =
  Utils.Render.statement_env (Code.print s.stmt) Ainput.print_value s.env

module Rinput = Utils.Text.Unit
module Routput = Utils.Text.Unit
module Foutput = Ainput

module Machine = struct
  module Code = Code
  module Ainput = Ainput
  module Rinput = Rinput
  module Routput = Routput
  module Foutput = Foutput

  type nonrec state = state

  let make = make
  let step = step
  let snapshot = snapshot
  let restore = restore
  let render = render
end
